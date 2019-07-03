use crate::{
    backtrace::Backtrace,
    codegen::{AsyncFn, Function, InvokerFn},
    context::Context,
    logger,
    registry::Registry,
    rpc::{
        client::FunctionRpc, status_result::Status, streaming_message::Content,
        FunctionLoadRequest, FunctionLoadResponse, InvocationRequest, InvocationResponse,
        StartStream, StatusResult, StreamingMessage, WorkerInitResponse, WorkerStatusRequest,
        WorkerStatusResponse,
    },
};
use http::{
    uri::{Authority, Parts, Scheme, Uri},
    Request as HttpRequest,
};
use log::error;
use std::cell::RefCell;
use std::panic::{catch_unwind, set_hook, AssertUnwindSafe, PanicInfo};
use tokio_threadpool::blocking;
use tower_grpc::Request;
use tower_hyper::{
    client::Builder as HttpBuilder,
    util::{Connector, Destination, HttpConnector},
    Connect,
};
use tower_service::Service;
use tower_util::MakeService;

use futures01::{future::poll_fn, sync::mpsc::unbounded, Async, Future, Poll, Stream};

pub type Sender = futures01::sync::mpsc::UnboundedSender<StreamingMessage>;

// TODO: replace with tower-request-modifier when published (see: https://github.com/tower-rs/tower-http/issues/24)
struct HttpOriginService<T> {
    inner: T,
    scheme: Scheme,
    authority: Authority,
}

impl<T> HttpOriginService<T> {
    pub fn new(inner: T, uri: Uri) -> Self {
        let parts = Parts::from(uri);

        HttpOriginService {
            inner,
            scheme: parts.scheme.unwrap(),
            authority: parts.authority.unwrap(),
        }
    }
}

impl<T, B> Service<HttpRequest<B>> for HttpOriginService<T>
where
    T: Service<HttpRequest<B>>,
{
    type Response = T::Response;
    type Error = T::Error;
    type Future = T::Future;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.inner.poll_ready()
    }

    fn call(&mut self, req: HttpRequest<B>) -> Self::Future {
        let (mut head, body) = req.into_parts();
        let mut parts = Parts::from(head.uri);

        parts.authority = Some(self.authority.clone());
        parts.scheme = Some(self.scheme.clone());

        head.uri = Uri::from_parts(parts).expect("valid uri");

        self.inner.call(HttpRequest::from_parts(head, body))
    }
}

struct ContextFuture<F> {
    inner: F,
    invocation_id: String,
    function_id: String,
    function_name: &'static str,
    sender: Sender,
}

impl<F> ContextFuture<F> {
    pub fn new(
        inner: F,
        invocation_id: String,
        function_id: String,
        function_name: &'static str,
        sender: Sender,
    ) -> Self {
        ContextFuture {
            inner,
            invocation_id,
            function_id,
            function_name,
            sender,
        }
    }
}

// TODO: migrate this to std::future::Future when Tokio supports it
impl<F: Future<Item = InvocationResponse>> Future for ContextFuture<F> {
    type Item = ();
    type Error = F::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let _guard = Context::set(&self.invocation_id, &self.function_id, self.function_name);

        let res = match catch_unwind(AssertUnwindSafe(|| self.inner.poll())) {
            Ok(p) => match p? {
                Async::Ready(res) => res,
                Async::NotReady => return Ok(Async::NotReady),
            },
            Err(_) => InvocationResponse {
                invocation_id: self.invocation_id.clone(),
                result: Some(StatusResult {
                    status: Status::Failure as i32,
                    result: "Azure Function panicked: see log for more information.".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            },
        };

        self.sender
            .unbounded_send(StreamingMessage {
                content: Some(Content::InvocationResponse(res)),
                ..Default::default()
            })
            .expect("failed to send invocation response");

        Ok(Async::Ready(()))
    }
}

pub struct Worker;

impl Worker {
    pub fn run(host: &str, port: u16, worker_id: &str, mut registry: Registry<'static>) {
        let host_uri: Uri = format!("http://{0}:{1}", host, port).parse().unwrap();
        let (sender, receiver) = unbounded::<StreamingMessage>();

        // Start by sending a start stream message to the channel
        // This will be sent to the host upon connection
        sender
            .unbounded_send(StreamingMessage {
                content: Some(Content::StartStream(StartStream {
                    worker_id: worker_id.to_owned(),
                })),
                ..Default::default()
            })
            .unwrap();

        let run = Connect::with_builder(
            Connector::new(HttpConnector::new(1)),
            HttpBuilder::new().http2_only(true).clone(),
        )
        .make_service(Destination::try_from_uri(host_uri.clone()).unwrap())
        .map(move |conn| FunctionRpc::new(HttpOriginService::new(conn, host_uri)))
        .map_err(|e| panic!("failed to connect to host: {}", e))
        .and_then(|mut client| {
            client
                .event_stream(Request::new(
                    receiver.map_err(|_| panic!("failed to receive from channel")),
                ))
                .map_err(|e| panic!("failed to start event stream: {}", e))
        })
        .and_then(move |stream| {
            stream
                .into_inner()
                .into_future()
                .map_err(|(e, _)| panic!("failed to read worker init request: {}", e))
                .and_then(move |(init_req, stream)| {
                    Worker::handle_worker_init_request(
                        sender.clone(),
                        init_req.expect("expected a worker init request"),
                    );

                    stream
                        .for_each(move |req| {
                            Worker::handle_request(&mut registry, sender.clone(), req);
                            Ok(())
                        })
                        .map_err(|e| panic!("fail to read request: {}", e))
                })
        });

        tokio::run(run);
    }

    fn handle_worker_init_request(sender: Sender, req: StreamingMessage) {
        match req.content {
            Some(Content::WorkerInitRequest(req)) => {
                println!(
                    "Connected to Azure Functions host version {}.",
                    req.host_version
                );

                // TODO: use the level requested by the Azure functions host
                log::set_boxed_logger(Box::new(logger::Logger::new(
                    log::Level::Info,
                    sender.clone(),
                )))
                .expect("failed to set the global logger instance");

                set_hook(Box::new(Worker::handle_panic));

                log::set_max_level(log::LevelFilter::Trace);

                sender
                    .unbounded_send(StreamingMessage {
                        content: Some(Content::WorkerInitResponse(WorkerInitResponse {
                            worker_version: env!("CARGO_PKG_VERSION").to_owned(),
                            result: Some(StatusResult {
                                status: Status::Success as i32,
                                ..Default::default()
                            }),
                            ..Default::default()
                        })),
                        ..Default::default()
                    })
                    .unwrap();
            }
            _ => panic!("expected a worker init request message from the host"),
        };
    }

    fn handle_request(registry: &mut Registry<'static>, sender: Sender, req: StreamingMessage) {
        match req.content {
            Some(Content::FunctionLoadRequest(req)) => {
                Worker::handle_function_load_request(registry, sender, req)
            }
            Some(Content::InvocationRequest(req)) => {
                Worker::handle_invocation_request(registry, sender, req)
            }
            Some(Content::WorkerStatusRequest(req)) => {
                Worker::handle_worker_status_request(sender, req)
            }
            Some(Content::FileChangeEventRequest(_)) => {}
            Some(Content::InvocationCancel(_)) => {}
            Some(Content::FunctionEnvironmentReloadRequest(_)) => {}
            _ => panic!("unexpected message from host: {:?}.", req),
        };
    }

    fn handle_function_load_request(
        registry: &mut Registry<'static>,
        sender: Sender,
        req: FunctionLoadRequest,
    ) {
        let mut result = StatusResult::default();

        match req.metadata.as_ref() {
            Some(metadata) => {
                if registry.register(&req.function_id, &metadata.name) {
                    result.status = Status::Success as i32;
                } else {
                    result.status = Status::Failure as i32;
                    result.result = format!("Function '{}' does not exist.", metadata.name);
                }
            }
            None => {
                result.status = Status::Failure as i32;
                result.result = "Function load request metadata is missing.".to_string();
            }
        };

        sender
            .unbounded_send(StreamingMessage {
                content: Some(Content::FunctionLoadResponse(FunctionLoadResponse {
                    function_id: req.function_id,
                    result: Some(result),
                    ..Default::default()
                })),
                ..Default::default()
            })
            .expect("failed to send function load response");
    }

    fn handle_invocation_request(
        registry: &Registry<'static>,
        sender: Sender,
        req: InvocationRequest,
    ) {
        if let Some(func) = registry.get(&req.function_id) {
            Worker::invoke_function(func, sender, req);
            return;
        }

        let error = format!("Function with id '{}' does not exist.", req.function_id);

        sender
            .unbounded_send(StreamingMessage {
                content: Some(Content::InvocationResponse(InvocationResponse {
                    invocation_id: req.invocation_id,
                    result: Some(StatusResult {
                        status: Status::Failure as i32,
                        result: error,
                        ..Default::default()
                    }),
                    ..Default::default()
                })),
                ..Default::default()
            })
            .expect("failed to send invocation response");
    }

    fn handle_worker_status_request(sender: Sender, _: WorkerStatusRequest) {
        sender
            .unbounded_send(StreamingMessage {
                content: Some(Content::WorkerStatusResponse(WorkerStatusResponse {})),
                ..Default::default()
            })
            .expect("failed to send worker status response");
    }

    fn invoke_function(func: &'static Function, sender: Sender, req: InvocationRequest) {
        match func
            .invoker
            .as_ref()
            .expect("function must have an invoker")
            .invoker_fn
        {
            InvokerFn::Sync(invoker_fn) => {
                // `poll_fn` takes FnMut and `blocking` takes FnOnce
                // Wrap the request with a RefCell so we can move the request to the invoked function
                let id = req.invocation_id.clone();
                let func_id = req.function_id.clone();
                let req = RefCell::new(Some(req));

                tokio::spawn(ContextFuture::new(
                    poll_fn(move || {
                        blocking(|| {
                            invoker_fn.expect("invoker must have a callback")(
                                req.replace(None).expect("only a single call to invoker"),
                            )
                        })
                    })
                    .map_err(|_| ()),
                    id,
                    func_id,
                    &func.name,
                    sender,
                ));
            }
            InvokerFn::Async(invoker_fn) => {
                Worker::invoke_function_async(
                    invoker_fn.expect("invoker must have a callback"),
                    func,
                    sender,
                    req,
                );
            }
        };
    }

    #[cfg(feature = "unstable")]
    fn invoke_function_async(
        invoker_fn: AsyncFn,
        func: &'static Function,
        sender: Sender,
        req: InvocationRequest,
    ) {
        use futures::future::{FutureExt, TryFutureExt};

        let id = req.invocation_id.clone();
        let func_id = req.function_id.clone();

        tokio::spawn(ContextFuture::new(
            invoker_fn(req).unit_error().compat(),
            id,
            func_id,
            &func.name,
            sender,
        ));
    }

    #[cfg(not(feature = "unstable"))]
    fn invoke_function_async(_: AsyncFn, _: &'static Function, _: Sender, _: InvocationRequest) {
        unimplemented!()
    }

    fn handle_panic(info: &PanicInfo) {
        let backtrace = Backtrace::new();
        match info.location() {
            Some(location) => {
                error!(
                    "Azure Function '{}' panicked with '{}', {}:{}:{}{}",
                    crate::context::CURRENT.with(|c| c.borrow().function_name),
                    info.payload()
                        .downcast_ref::<&str>()
                        .cloned()
                        .unwrap_or_else(|| info
                            .payload()
                            .downcast_ref::<String>()
                            .map(String::as_str)
                            .unwrap_or("")),
                    location.file(),
                    location.line(),
                    location.column(),
                    backtrace
                );
            }
            None => {
                error!(
                    "Azure Function '{}' panicked with '{}'{}",
                    crate::context::CURRENT.with(|c| c.borrow().function_name),
                    info.payload()
                        .downcast_ref::<&str>()
                        .cloned()
                        .unwrap_or_else(|| info
                            .payload()
                            .downcast_ref::<String>()
                            .map(String::as_str)
                            .unwrap_or("")),
                    backtrace
                );
            }
        };
    }
}
