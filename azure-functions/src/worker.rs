use crate::{
    backtrace::Backtrace,
    codegen::{Function, InvokerFn},
    context::Context,
    logger,
    registry::Registry,
    rpc::{
        client::FunctionRpcClient, status_result::Status, streaming_message::Content,
        FunctionLoadRequest, FunctionLoadResponse, InvocationRequest, InvocationResponse,
        StartStream, StatusResult, StreamingMessage, WorkerInitResponse, WorkerStatusRequest,
        WorkerStatusResponse,
    },
};
use futures::{channel::mpsc::unbounded, future::FutureExt, stream::StreamExt};
use http::uri::Uri;
use log::error;
use std::{
    cell::RefCell,
    future::Future,
    panic::{catch_unwind, set_hook, AssertUnwindSafe, PanicInfo},
    pin::Pin,
    task::Poll,
};
use tokio::future::poll_fn;
use tokio_executor::threadpool::blocking;
use tonic::Request;

pub type Sender = futures::channel::mpsc::UnboundedSender<Result<StreamingMessage, tonic::Status>>;

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

impl<F: Future<Output = InvocationResponse> + Unpin> Future for ContextFuture<F> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        let _guard = Context::set(&self.invocation_id, &self.function_id, self.function_name);

        let res = match catch_unwind(AssertUnwindSafe(|| self.inner.poll_unpin(cx))) {
            Ok(p) => match p {
                Poll::Ready(res) => res,
                Poll::Pending => return Poll::Pending,
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
            .unbounded_send(Ok(StreamingMessage {
                content: Some(Content::InvocationResponse(res)),
                ..Default::default()
            }))
            .expect("failed to send invocation response");

        Poll::Ready(())
    }
}

pub struct Worker;

impl Worker {
    pub fn run(host: &str, port: u16, worker_id: &str, mut registry: Registry<'static>) {
        let host_uri: Uri = format!("http://{0}:{1}", host, port).parse().unwrap();
        let (sender, receiver) = unbounded::<Result<StreamingMessage, tonic::Status>>();

        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let mut client = FunctionRpcClient::connect(host_uri)
                .map_err(|e| panic!("failed to connect to host: {}", e))
                .unwrap();

            // Start by sending a start stream message to the channel
            // This will be sent to the host upon connection
            sender
                .unbounded_send(Ok(StreamingMessage {
                    content: Some(Content::StartStream(StartStream {
                        worker_id: worker_id.to_owned(),
                    })),
                    ..Default::default()
                }))
                .unwrap();

            let mut stream = client
                .event_stream(Request::new(receiver))
                .await
                .map_err(|e| panic!("failed to start event stream: {}", e))
                .unwrap()
                .into_inner();

            let init_req = stream
                .next()
                .await
                .expect("expected a worker init request")
                .map_err(|e| panic!("failed to read event stream response: {}", e))
                .unwrap();

            Worker::handle_worker_init_request(sender.clone(), init_req).await;

            stream
                .for_each(move |req| {
                    Worker::handle_request(
                        &mut registry,
                        sender.clone(),
                        req.expect("expected a request"),
                    );
                    futures::future::ready(())
                })
                .await;
        });
    }

    async fn handle_worker_init_request(sender: Sender, req: StreamingMessage) {
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
                    .unbounded_send(Ok(StreamingMessage {
                        content: Some(Content::WorkerInitResponse(WorkerInitResponse {
                            worker_version: env!("CARGO_PKG_VERSION").to_owned(),
                            result: Some(StatusResult {
                                status: Status::Success as i32,
                                ..Default::default()
                            }),
                            ..Default::default()
                        })),
                        ..Default::default()
                    }))
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
            .unbounded_send(Ok(StreamingMessage {
                content: Some(Content::FunctionLoadResponse(FunctionLoadResponse {
                    function_id: req.function_id,
                    result: Some(result),
                    ..Default::default()
                })),
                ..Default::default()
            }))
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
            .unbounded_send(Ok(StreamingMessage {
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
            }))
            .expect("failed to send invocation response");
    }

    fn handle_worker_status_request(sender: Sender, _: WorkerStatusRequest) {
        sender
            .unbounded_send(Ok(StreamingMessage {
                content: Some(Content::WorkerStatusResponse(WorkerStatusResponse {})),
                ..Default::default()
            }))
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
                    poll_fn(move |_| {
                        blocking(|| {
                            invoker_fn.expect("invoker must have a callback")(
                                req.replace(None).expect("only a single call to invoker"),
                            )
                        })
                    })
                    .map(|r| r.expect("expected a response")),
                    id,
                    func_id,
                    &func.name,
                    sender,
                ));
            }
            InvokerFn::Async(invoker_fn) => {
                let id = req.invocation_id.clone();
                let func_id = req.function_id.clone();

                tokio::spawn(ContextFuture::new(
                    invoker_fn.expect("invoker must have a callback")(req),
                    id,
                    func_id,
                    &func.name,
                    sender,
                ));
            }
        };
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
