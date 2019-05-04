use crate::{
    backtrace::Backtrace,
    codegen::Function,
    logger,
    registry::Registry,
    rpc::{
        client::FunctionRpc, status_result::Status, streaming_message::Content,
        FunctionLoadRequest, FunctionLoadResponse, InvocationRequest, InvocationResponse,
        StartStream, StatusResult, StreamingMessage, WorkerInitResponse, WorkerStatusRequest,
        WorkerStatusResponse,
    },
};
use clap::{App, Arg, ArgMatches, SubCommand};
use futures::{future::lazy, sync::mpsc::unbounded, Future, Poll, Stream};
use log::error;
use std::cell::RefCell;
use std::net::{IpAddr, SocketAddr};
use std::panic::{catch_unwind, set_hook, AssertUnwindSafe, PanicInfo};
use tokio::{
    executor::DefaultExecutor,
    net::tcp::{ConnectFuture, TcpStream},
};
use tower_grpc::Request;
use tower_h2::client::Connect;
use tower_service::Service;
use tower_util::MakeService;

const UNKNOWN: &str = "<unknown>";

thread_local!(static FUNCTION_NAME: RefCell<&'static str> = RefCell::new(UNKNOWN));

type Sender = futures::sync::mpsc::UnboundedSender<StreamingMessage>;

struct HttpService(IpAddr, u16);

impl Service<()> for HttpService {
    type Response = TcpStream;
    type Error = ::std::io::Error;
    type Future = ConnectFuture;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        Ok(().into())
    }

    fn call(&mut self, _: ()) -> Self::Future {
        TcpStream::connect(&SocketAddr::new(self.0, self.1))
    }
}

pub struct Run<'a> {
    pub host: &'a str,
    pub port: u16,
    pub worker_id: &'a str,
}

impl<'a> Run<'a> {
    pub fn create_subcommand<'b>() -> App<'a, 'b> {
        SubCommand::with_name("run")
            .about("Runs the Rust language worker.")
            .arg(
                Arg::with_name("host")
                    .long("host")
                    .value_name("HOST")
                    .help("The hostname of the Azure Functions Host.")
                    .required(true),
            )
            .arg(
                Arg::with_name("port")
                    .long("port")
                    .value_name("PORT")
                    .help("The port of the Azure Functions Host.")
                    .required(true),
            )
            .arg(
                Arg::with_name("worker_id")
                    .long("workerId")
                    .value_name("WORKER_ID")
                    .help("The worker ID to use when registering with the Azure Functions Host.")
                    .required(true),
            )
            .arg(
                Arg::with_name("request_id")
                    .long("requestId")
                    .value_name("REQUEST_ID")
                    .help("The request ID to use when communicating with the Azure Functions Host.")
                    .hidden(true)
                    .required(true),
            )
            .arg(
                Arg::with_name("max_message_length")
                    .long("grpcMaxMessageLength")
                    .value_name("MAXIMUM")
                    .help("The maximum message length to use for gRPC messages."),
            )
    }

    pub fn execute(&self, mut registry: Registry<'static>) -> Result<(), String> {
        ctrlc::set_handler(|| {}).expect("failed setting SIGINT handler");

        let uri = format!("http://{0}:{1}", self.host, self.port);
        let (sender, receiver) = unbounded::<StreamingMessage>();

        // Start by sending a start stream message to the channel
        // This will be sent to the host upon connection
        sender
            .unbounded_send(StreamingMessage {
                content: Some(Content::StartStream(StartStream {
                    worker_id: self.worker_id.to_owned(),
                })),
                ..Default::default()
            })
            .unwrap();

        println!("Connecting to Azure Functions host at {}", uri);

        let run = Connect::new(
            HttpService(self.host.parse().unwrap(), self.port),
            Default::default(),
            DefaultExecutor::current(),
        )
        .make_service(())
        .map(move |conn| {
            FunctionRpc::new(
                tower_request_modifier::Builder::new()
                    .set_origin(uri)
                    .build(conn)
                    .unwrap(),
            )
        })
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
                    Run::handle_worker_init_request(
                        sender.clone(),
                        init_req.expect("expected a worker init request"),
                    );

                    stream
                        .for_each(move |req| {
                            Run::handle_request(&mut registry, sender.clone(), req);
                            Ok(())
                        })
                        .map_err(|e| panic!("fail to read request: {}", e))
                })
        });

        tokio::run(run);

        Ok(())
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

                set_hook(Box::new(Run::handle_panic));

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
                Run::handle_function_load_request(registry, sender, req)
            }
            Some(Content::InvocationRequest(req)) => {
                Run::handle_invocation_request(registry, sender, req)
            }
            Some(Content::WorkerStatusRequest(req)) => {
                Run::handle_worker_status_request(sender, req)
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
            tokio::spawn(lazy(move || {
                Run::invoke_function(func, sender, req);
                Ok(())
            }));
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
        // Set the function name in TLS
        FUNCTION_NAME.with(|n| {
            *n.borrow_mut() = &func.name;
        });

        // Set the invocation ID in TLS
        logger::INVOCATION_ID.with(|id| {
            id.borrow_mut().replace_range(.., &req.invocation_id);
        });

        let response = match catch_unwind(AssertUnwindSafe(|| {
            (func
                .invoker
                .as_ref()
                .expect("function must have an invoker"))(&func.name, req)
        })) {
            Ok(res) => res,
            Err(_) => logger::INVOCATION_ID.with(|id| InvocationResponse {
                invocation_id: id.borrow().clone(),
                result: Some(StatusResult {
                    status: Status::Failure as i32,
                    result: "Azure Function panicked: see log for more information.".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        };

        // Clear the function name from TLS
        FUNCTION_NAME.with(|n| {
            *n.borrow_mut() = UNKNOWN;
        });

        // Clear the invocation ID from TLS
        logger::INVOCATION_ID.with(|id| {
            id.borrow_mut().clear();
        });

        sender
            .unbounded_send(StreamingMessage {
                content: Some(Content::InvocationResponse(response)),
                ..Default::default()
            })
            .expect("failed to send invocation response");
    }

    fn handle_panic(info: &PanicInfo) {
        let backtrace = Backtrace::new();
        match info.location() {
            Some(location) => {
                error!(
                    "Azure Function '{}' panicked with '{}', {}:{}:{}{}",
                    FUNCTION_NAME.with(|f| *f.borrow()),
                    info.payload()
                        .downcast_ref::<&str>()
                        .cloned()
                        .unwrap_or_else(|| info
                            .payload()
                            .downcast_ref::<String>()
                            .map(String::as_str)
                            .unwrap_or(UNKNOWN)),
                    location.file(),
                    location.line(),
                    location.column(),
                    backtrace
                );
            }
            None => {
                error!(
                    "Azure Function '{}' panicked with '{}'{}",
                    FUNCTION_NAME.with(|f| *f.borrow()),
                    info.payload()
                        .downcast_ref::<&str>()
                        .cloned()
                        .unwrap_or_else(|| info
                            .payload()
                            .downcast_ref::<String>()
                            .map(String::as_str)
                            .unwrap_or(UNKNOWN)),
                    backtrace
                );
            }
        };
    }
}

impl<'a> From<&'a ArgMatches<'a>> for Run<'a> {
    fn from(args: &'a ArgMatches<'a>) -> Self {
        Run {
            host: args.value_of("host").expect("A host is required."),
            port: args
                .value_of("port")
                .map(|port| port.parse::<u16>().expect("Invalid port number"))
                .expect("A port number is required."),
            worker_id: args
                .value_of("worker_id")
                .expect("A worker id is required."),
        }
    }
}
