use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;
use serde_repr::Deserialize_repr;

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct HistoryEvent {
    pub event_type: EventType,

    pub event_id: i32,

    pub is_played: bool,

    pub timestamp: DateTime<Utc>,

    #[serde(skip)]
    pub is_processed: bool,

    // Used by: EventRaised, ExecutionStarted, SubOrchestrationInstanceCreated, TaskScheduled
    pub name: Option<String>,

    // Used by: EventRaised, ExecutionStarted, SubOrchestrationInstanceCreated, TaskScheduled
    pub input: Option<Value>,

    // Used by: SubOrchestrationInstanceCompleted, TaskCompleted
    pub result: Option<String>,

    // Used by: SubOrchestrationInstanceCompleted , SubOrchestrationInstanceFailed, TaskCompleted,TaskFailed
    pub task_scheduled_id: Option<i32>,

    // Used by: SubOrchestrationInstanceCreated
    pub instance_id: Option<String>,

    // Used by: SubOrchestrationInstanceFailed, TaskFailed
    pub reason: Option<String>,

    // Used by: SubOrchestrationInstanceFailed,TaskFailed
    pub details: Option<String>,

    // Used by: TimerCreated, TimerFired
    pub fire_at: Option<DateTime<Utc>>,

    // Used by: TimerFired
    pub timer_id: Option<i32>,
}

#[derive(Debug, Copy, Clone, Deserialize_repr, PartialEq)]
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
