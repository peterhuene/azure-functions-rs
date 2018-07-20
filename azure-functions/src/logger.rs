use futures::{Future, Sink};
use log::{Level, Log, Metadata, Record};
use rpc::{protocol, Sender};

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
        if self.enabled(record.metadata()) {
            let mut message = protocol::StreamingMessage::new();
            {
                let response = message.mut_rpc_log();
                response.level = match record.level() {
                    Level::Trace => protocol::RpcLog_Level::Trace,
                    Level::Debug => protocol::RpcLog_Level::Debug,
                    Level::Info => protocol::RpcLog_Level::Information,
                    Level::Warn => protocol::RpcLog_Level::Warning,
                    Level::Error => protocol::RpcLog_Level::Error,
                };
                response.message = record.args().to_string();
            }

            // Not happy with this clone of the sender upon each logged message :/
            self.sender.clone().send(message).wait().unwrap();
        }
    }

    fn flush(&self) {}
}
