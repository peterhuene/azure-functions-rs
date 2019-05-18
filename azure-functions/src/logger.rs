use crate::rpc::{rpc_log, streaming_message::Content, RpcLog, StreamingMessage};
use log::{Level, Log, Metadata, Record};
use std::cell::RefCell;

thread_local!(pub static INVOCATION_ID: RefCell<String> = RefCell::new(String::new()));

type Sender = futures::sync::mpsc::UnboundedSender<StreamingMessage>;

pub struct Logger {
    level: Level,
    sender: Sender,
}

impl Logger {
    pub fn new(level: Level, sender: Sender) -> Logger {
        Logger { level, sender }
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

        let mut event = RpcLog {
            level: match record.level() {
                Level::Trace => rpc_log::Level::Trace,
                Level::Debug => rpc_log::Level::Debug,
                Level::Info => rpc_log::Level::Information,
                Level::Warn => rpc_log::Level::Warning,
                Level::Error => rpc_log::Level::Error,
            } as i32,
            message: record.args().to_string(),
            ..Default::default()
        };

        INVOCATION_ID.with(|id| {
            let id = id.borrow();
            if !id.is_empty() {
                event.invocation_id = id.clone();
            }
        });

        self.sender
            .unbounded_send(StreamingMessage {
                content: Some(Content::RpcLog(event)),
                ..Default::default()
            })
            .unwrap_or(());
    }

    fn flush(&self) {}
}
