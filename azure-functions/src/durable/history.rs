use chrono::{DateTime, FixedOffset};
use serde::Deserialize;
use serde_json::Value;
use serde_repr::Deserialize_repr;

// TODO refactor this to make enum HistoryEvent that for each value it will have its own struct
// enum HistoryEvent { ExecutionStarted(ExecutionStartedEvent), ... }
// serde now doesn't support elegant conversion from json to tagged enums with custom tag value out of the box.
// i.e conversion { EventType = 0, EventId = 1, ...} => ExecutionStarted(ExecutionStartedEvent)
// in future we can implement manual translation

#[doc(hidden)]
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct HistoryEvent {
    pub(crate) event_type: EventType,

    pub(crate) event_id: i32,

    #[serde(alias = "IsPlayed")]
    pub(crate) played: bool,

    pub(crate) timestamp: DateTime<FixedOffset>,

    #[serde(default, alias = "IsProcessed")]
    pub(crate) processed: bool,

    // EventRaised, ExecutionStarted, SubOrchestrationInstanceCreated, TaskScheduled
    pub(crate) name: Option<String>,

    // EventRaised, ExecutionStarted, SubOrchestrationInstanceCreated, TaskScheduled
    pub(crate) input: Option<Value>,

    //SubOrchestrationInstanceCompleted, TaskCompleted
    pub(crate) result: Option<Value>,

    // SubOrchestrationInstanceCompleted , SubOrchestrationInstanceFailed, TaskCompleted,TaskFailed
    pub(crate) task_scheduled_id: Option<i32>,

    // SubOrchestrationInstanceCreated
    pub(crate) instance_id: Option<String>,

    //SubOrchestrationInstanceFailed, TaskFailed
    pub(crate) reason: Option<String>,

    // SubOrchestrationInstanceFailed,TaskFailed
    pub(crate) details: Option<String>,

    //TimerCreated, TimerFired
    pub(crate) fire_at: Option<DateTime<FixedOffset>>,

    //TimerFired
    pub(crate) timer_id: Option<i32>,
}

#[doc(hidden)]
#[derive(Debug, Clone, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub(crate) enum EventType {
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
    // not supported in js
    EventRaised = 15,
    // not supported in js
    ContinueAsNew = 16,
    // not supported in js
    GenericEvent = 17,
    // not supported in js
    HistoryState = 18,
}
