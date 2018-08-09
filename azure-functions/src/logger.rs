use futures::{Future, Sink};
use log::{Level, Log, Metadata, Record};
use rpc::{protocol, Sender};
use std::cell::RefCell;

thread_local!(pub static INVOCATION_ID: RefCell<String> = RefCell::new(String::new()));

pub struct Logger {
    level: Level,
    sender: Sender,
}

impl Logger {
    pub fn new(level: Level, sender: Sender) -> Logger {
        Logger {
            level: level,
            sender: sender,
        }
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let mut event = protocol::RpcLog::new();
        event.set_level(match record.level() {
            Level::Trace => protocol::RpcLog_Level::Trace,
            Level::Debug => protocol::RpcLog_Level::Debug,
            Level::Info => protocol::RpcLog_Level::Information,
            Level::Warn => protocol::RpcLog_Level::Warning,
            Level::Error => protocol::RpcLog_Level::Error,
        });
        event.set_message(record.args().to_string());

        INVOCATION_ID.with(|id| {
            let id = id.borrow();
            if !id.is_empty() {
                event.set_invocation_id(id.clone());
            }
        });

        let mut message = protocol::StreamingMessage::new();
        message.set_rpc_log(event);
        self.sender.clone().send(message).wait().unwrap();
    }

    fn flush(&self) {}
}
