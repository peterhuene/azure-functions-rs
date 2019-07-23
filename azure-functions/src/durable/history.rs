use chrono::{DateTime, FixedOffset};
use serde::Deserialize;
use serde_repr::Deserialize_repr;

#[doc(hidden)]
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct HistoryEvent {
    pub(crate) event_type: EventType,
    pub(crate) event_id: i32,
    pub(crate) is_played: bool,
    pub(crate) timestamp: DateTime<FixedOffset>,
    #[serde(default)]
    pub(crate) is_processed: bool,
    pub(crate) name: Option<String>,
    pub(crate) input: Option<String>,
}

#[derive(Debug, Clone, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub enum EventType {
    ExecutionStarted = 0,
    ExecutionCompleted = 1,
    ExecutionFailed = 2,
    ExecutionTerminated = 3,
    TaskScheduled = 4,
    TaskCompleted = 5,
    TaskFailed = 6,
    SubOrchestrationInstanceCreated = 7,
    SubOrchestrationInstanceCompleted = 8,
    SubOrchestrationInstanceFailed = 9,
    TimerCreated = 10,
    TimerFired = 11,
    OrchestratorStarted = 12,
    OrchestratorCompleted = 13,
    EventSent = 14,
    EventRaised = 15,
    ContinueAsNew = 16,
    GenericEvent = 17,
    HistoryState = 18,
}
