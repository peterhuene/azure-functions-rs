use crate::codegen::Function;
use crate::logger;
use crate::registry::Registry;
use azure_functions_shared::rpc::protocol;
use futures::future::{lazy, ok};
use futures::sync::mpsc;
use futures::{Future, Sink, Stream};
use grpcio::{ChannelBuilder, ClientDuplexReceiver, EnvBuilder, WriteFlags};
use log::{self, error};
use std::cell::RefCell;
use std::panic::{self, AssertUnwindSafe};
use std::sync::Arc;
use std::thread;
use tokio_threadpool::ThreadPool;

pub type Sender = mpsc::Sender<protocol::StreamingMessage>;
type Receiver = ClientDuplexReceiver<protocol::StreamingMessage>;

const UNKNOWN: &str = "<unknown>";

thread_local!(static FUNCTION_NAME: RefCell<&'static str> = RefCell::new(UNKNOWN));

pub struct Client {
    worker_id: String,
    max_message_len: Option<i32>,
    client: Option<protocol::FunctionRpcClient>, // We must store the client to ensure the underlying channel isn't dropped
    sender: Option<Sender>,
    receiver: Option<Receiver>,
    host_version: Option<String>,
}

impl Client {
    pub fn new(worker_id: String, max_message_len: Option<i32>) -> Client {
        Client {
            worker_id,
            max_message_len,
            client: None,
            sender: None,
            receiver: None,
            host_version: None,
        }
    }

    pub fn host_version(&self) -> Option<&str> {
        self.host_version.as_ref().map(String::as_str)
    }

    pub fn sender(&self) -> Option<Sender> {
        self.sender.clone()
    }

    pub fn connect(mut self, host: &str, port: u32) -> impl Future<Item = Client, Error = ()> {
        let mut channel = ChannelBuilder::new(Arc::new(EnvBuilder::new().build()));

        if let Some(len) = self.max_message_len {
            if len > 0 {
                channel = channel
                    .max_receive_message_len(len)
                    .max_send_message_len(len);
            }
        }

        let (rpc_tx, rpc_rx) = self
            .client
            .get_or_insert(protocol::FunctionRpcClient::new(
                channel.connect(&format!("{}:{}", host, port)),
            ))
            .event_stream()
            .unwrap();

        let (tx, rx) = mpsc::channel(1);

        self.sender = Some(tx);
        self.receiver = Some(rpc_rx);

        thread::spawn(move || {
            let mut rx = rx;
            let mut rpc_tx = rpc_tx;

            while let (Some(message), r) = rx.into_future().wait().unwrap() {
                rpc_tx = rpc_tx
                    .send((message, WriteFlags::default()))
                    .wait()
                    .expect("failed to send message to host");
                rx = r;
            }
        });

        let mut message = protocol::StreamingMessage::new();
        message.mut_start_stream().worker_id = self.worker_id.to_owned();

        self.send(message)
            .and_then(Client::read)
            .and_then(|(mut c, msg)| {
                let msg = msg.expect("host disconnected during worker initialization");

                if !msg.has_worker_init_request() {
                    panic!("expected a worker init request, but received: {:?}.", msg);
                }

                c.host_version = Some(msg.get_worker_init_request().host_version.clone());

                let mut msg = protocol::StreamingMessage::new();
                {
                    let worker_init_res = msg.mut_worker_init_response();
                    worker_init_res.worker_version = env!("CARGO_PKG_VERSION").to_owned();
                    let result = worker_init_res.mut_result();
                    result.status = protocol::StatusResult_Status::Success;
                }

                c.send(msg)
            })
    }

    pub fn send(
        mut self,
        message: protocol::StreamingMessage,
    ) -> impl Future<Item = Client, Error = ()> {
        self.sender
            .take()
            .unwrap()
            .send(message)
            .map_err(|err| panic!("failed to send message: {:?}.", err))
            .and_then(move |sender| {
                self.sender = Some(sender);
                ok(self)
            })
    }

    pub fn read(
        mut self,
    ) -> impl Future<Item = (Client, Option<protocol::StreamingMessage>), Error = ()> {
        self.receiver
            .take()
            .unwrap()
            .into_future()
            .map_err(|(err, _)| panic!("failed to receive message: {:?}.", err))
            .and_then(move |(msg, r)| {
                self.receiver = Some(r);
                ok((self, msg))
            })
    }

    pub fn process_all_messages(
        mut self,
        mut registry: Registry<'static>,
    ) -> impl Future<Item = Client, Error = ()> {
        let pool = tokio_threadpool::ThreadPool::new();

        // TODO: use the level requested by the Azure functions host
        log::set_boxed_logger(Box::new(logger::Logger::new(
            log::Level::Trace,
            self.sender.clone().unwrap(),
        )))
        .expect("Failed to set the global logger instance");

        // At this point, translate any panics to error! macros to log with the host
        panic::set_hook(Box::new(|info| match info.location() {
            Some(location) => {
                error!(
                    "Azure Function '{}' panicked with '{}', {}:{}:{}",
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
                    location.column()
                );
            }
            None => {
                error!(
                    "Azure Function '{}' panicked with '{}'",
                    FUNCTION_NAME.with(|f| *f.borrow()),
                    info.payload()
                        .downcast_ref::<&str>()
                        .cloned()
                        .unwrap_or_else(|| info
                            .payload()
                            .downcast_ref::<String>()
                            .map(String::as_str)
                            .unwrap_or(UNKNOWN)),
                );
            }
        }));

        log::set_max_level(log::LevelFilter::Trace);

        loop {
            let (c, msg) = self.read().wait().expect("Failed to read message");
            self = c;

            if msg.is_none() {
                break;
            }

            let msg = msg.unwrap();
            if msg.has_worker_terminate() {
                break;
            }

            Client::handle_request(&mut registry, self.sender().unwrap(), msg, &pool);
        }

        pool.shutdown_on_idle().and_then(|_| ok(self))
    }

    fn handle_function_load_request(
        registry: &mut Registry<'static>,
        sender: Sender,
        req: protocol::FunctionLoadRequest,
    ) {
        let mut message = protocol::StreamingMessage::new();
        {
            let response = message.mut_function_load_response();
            response.function_id = req.function_id.clone();

            response.set_result(match req.metadata.as_ref() {
                Some(metadata) => {
                    let mut result = protocol::StatusResult::new();
                    if registry.register(&req.function_id, &metadata.name) {
                        result.status = protocol::StatusResult_Status::Success;
                    } else {
                        result.status = protocol::StatusResult_Status::Failure;
                        result.result = format!("Function '{}' does not exist.", metadata.name);
                    }
                    result
                }
                None => {
                    let mut result = protocol::StatusResult::new();
                    result.status = protocol::StatusResult_Status::Failure;
                    result.result = "Function load request metadata is missing.".to_string();
                    result
                }
            });
        }

        sender
            .send(message)
            .wait()
            .expect("Failed to send message to response thread");
    }

    fn invoke_function(
        func: &'static Function,
        sender: Sender,
        mut req: protocol::InvocationRequest,
    ) {
        // Set the function name in TLS
        FUNCTION_NAME.with(|n| {
            *n.borrow_mut() = &func.name;
        });

        // Set the invocation ID in TLS
        logger::INVOCATION_ID.with(|id| {
            id.borrow_mut().replace_range(.., &req.invocation_id);
        });

        let res = match panic::catch_unwind(AssertUnwindSafe(|| {
            (func
                .invoker
                .as_ref()
                .expect("function must have an invoker"))(&func.name, &mut req)
        })) {
            Ok(res) => res,
            Err(_) => {
                let mut res = protocol::InvocationResponse::new();
                res.set_invocation_id(req.invocation_id.clone());
                let mut result = protocol::StatusResult::new();
                result.status = protocol::StatusResult_Status::Failure;
                result.result =
                    "Azure Function panicked: see log for more information.".to_string();
                res.set_result(result);
                res
            }
        };

        // Clear the function name from TLS
        FUNCTION_NAME.with(|n| {
            *n.borrow_mut() = UNKNOWN;
        });

        // Clear the invocation ID from TLS
        logger::INVOCATION_ID.with(|id| {
            id.borrow_mut().clear();
        });

        let mut message = protocol::StreamingMessage::new();
        message.set_invocation_response(res);

        sender
            .send(message)
            .wait()
            .expect("Failed to send message to response thread");
    }

    fn handle_invocation_request(
        registry: &Registry<'static>,
        sender: Sender,
        req: protocol::InvocationRequest,
        pool: &ThreadPool,
    ) {
        if let Some(func) = registry.get(&req.function_id) {
            pool.spawn(lazy(move || {
                Client::invoke_function(func, sender, req);
                Ok(())
            }));
            return;
        }

        let mut res = protocol::InvocationResponse::new();
        res.set_invocation_id(req.invocation_id.clone());
        let mut result = protocol::StatusResult::new();
        result.status = protocol::StatusResult_Status::Failure;
        result.result = format!("Function with id '{}' does not exist.", req.function_id);
        res.set_result(result);

        let mut message = protocol::StreamingMessage::new();
        message.set_invocation_response(res);

        sender
            .send(message)
            .wait()
            .expect("Failed to send message to response thread");
    }

    fn handle_worker_status_request(sender: Sender, _req: protocol::WorkerStatusRequest) {
        let mut message = protocol::StreamingMessage::new();
        {
            message.mut_worker_status_response();
            // TODO: in the future, this message might have fields to set
        }

        sender
            .send(message)
            .wait()
            .expect("Failed to send message to response thread");
    }

    fn handle_request(
        registry: &mut Registry<'static>,
        sender: Sender,
        mut msg: protocol::StreamingMessage,
        pool: &ThreadPool,
    ) {
        if msg.has_function_load_request() {
            Client::handle_function_load_request(
                registry,
                sender,
                msg.take_function_load_request(),
            );
            return;
        }
        if msg.has_invocation_request() {
            Client::handle_invocation_request(
                registry,
                sender,
                msg.take_invocation_request(),
                pool,
            );
            return;
        }
        if msg.has_worker_status_request() {
            Client::handle_worker_status_request(sender, msg.take_worker_status_request());
            return;
        }
        if msg.has_file_change_event_request() {
            // Not supported (no-op)
            return;
        }
        if msg.has_invocation_cancel() {
            // Not supported (no-op)
            return;
        }
        if msg.has_function_environment_reload_request() {
            // Not supported (no-op)
            return;
        }

        panic!("Unexpected message from host: {:?}.", msg);
    }
}
