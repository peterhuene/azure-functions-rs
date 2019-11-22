use std::error::Error;
use std::fmt::{Display, Formatter};

/// Represents a Durable Functions HTTP client error.
#[derive(Debug, Clone, PartialEq)]
pub enum ClientError {
    /// Orchestration instance is in a failed or terminated state.
    InstanceFailedOrTerminated,
    /// Orchestration instance is in a completed or failed state.
    InstanceCompletedOrFailed,
    /// The orchestration instance was not found.
    InstanceNotFound,
    /// The specified entity was not found.
    EntityNotFound,
    /// The request contained invalid JSON data.
    BadRequest,
    /// The specified orchestrator function doesn't exist or the request contained invalid JSON data.
    BadCreateRequest,
    /// The request failed due to an exception while processing the request.
    InternalServerError,
    /// The error is a message.
    Message(String),
}

impl Display for ClientError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::InstanceFailedOrTerminated => write!(f, "theinstance failed or was terminated"),
            Self::InstanceCompletedOrFailed => write!(f, "instance completed or failed"),
            Self::InstanceNotFound => {
                write!(f, "instance doesn't exist or has not started running")
            }
            Self::EntityNotFound => write!(f, "entity type doesn't exist"),
            Self::BadRequest => write!(f, "request content was not valid JSON"),
            Self::BadCreateRequest => write!(f, "the specified orchestrator function doesn't exist, the specified instance ID was not valid, or request content was not valid JSON"),
            Self::InternalServerError => write!(f, "internal server error"),
            Self::Message(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for ClientError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
