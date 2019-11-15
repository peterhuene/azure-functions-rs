use crate::{
    rpc::{rpc_log, streaming_message::Content, RpcLog, StreamingMessage},
    worker::Sender,
};
use log::{Level, Log, Metadata, Record};

pub struct Logger {
    level: Level,
    sender: Sender,
}

impl Logger {
    pub fn new(level: Level, sender: Sender) -> Self {
        Self { level, sender }
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

        event.invocation_id = crate::context::CURRENT.with(|c| c.borrow().invocation_id.clone());

        self.sender
            .unbounded_send(StreamingMessage {
                content: Some(Content::RpcLog(event)),
                ..Default::default()
            })
            .unwrap_or(());
    }

    fn flush(&self) {}
}
