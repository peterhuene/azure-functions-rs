use crate::durable::{Action, EventType, HistoryEvent};
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{to_string, Value};

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct ExecutionResult {
    is_done: bool,
    actions: Vec<Vec<Action>>,
    output: Option<Value>,
    custom_status: Option<Value>,
    error: Option<String>,
}

#[doc(hidden)]
pub struct OrchestrationState {
    pub(crate) history: Vec<HistoryEvent>,
    result: ExecutionResult,
    started_index: usize,
    completed_index: Option<usize>,
}

impl OrchestrationState {
    pub(crate) fn new(history: Vec<HistoryEvent>) -> Self {
        let started_index = history
            .iter()
            .position(|event| event.event_type == EventType::OrchestratorStarted)
            .expect("failed to find orchestration started event");

        let completed_index = history[started_index..]
            .iter()
            .position(|event| event.event_type == EventType::OrchestratorCompleted)
            .map(|pos| pos + started_index);

        OrchestrationState {
            history,
            result: ExecutionResult::default(),
            started_index,
            completed_index,
        }
    }

    pub(crate) fn is_replaying(&self) -> bool {
        self.completed_index.is_some()
    }

    pub(crate) fn current_time(&self) -> DateTime<Utc> {
        self.history[self.started_index].timestamp
    }

    pub(crate) fn push_action(&mut self, action: Action) {
        if self.result.actions.is_empty() {
            self.result.actions.push(Vec::new());
        }

        self.result.actions.last_mut().unwrap().push(action);
    }

    pub(crate) fn set_output(&mut self, value: Value) {
        self.result.output = Some(value);
        self.result.is_done = true;
    }

    pub(crate) fn result(&self) -> String {
        to_string(&self.result).unwrap()
    }

    pub(crate) fn find_scheduled_task(
        &mut self,
        activity_name: &str,
    ) -> Option<(usize, &mut HistoryEvent)> {
        let index = self.history.iter().position(|event| {
            !event.is_processed
                && event.event_type == EventType::TaskScheduled
                && event.name == Some(activity_name.to_owned())
        })?;

        Some((index, &mut self.history[index]))
    }

    pub(crate) fn find_finished_task(
        &mut self,
        scheduled_index: usize,
    ) -> Option<(usize, &mut HistoryEvent)> {
        if scheduled_index + 1 >= self.history.len() {
            return None;
        }

        let id = self.history[scheduled_index].event_id;

        let index = self.history[scheduled_index + 1..]
            .iter()
            .position(|event| {
                (event.event_type == EventType::TaskCompleted
                    || event.event_type == EventType::TaskFailed)
                    && event.task_scheduled_id == Some(id)
            })
            .map(|p| p + scheduled_index + 1)?;

        Some((index, &mut self.history[index]))
    }

    pub(crate) fn update(&mut self, event_index: usize) {
        // Check for end of history
        if self.started_index + 1 >= self.history.len() || self.completed_index.is_none() {
            return;
        }

        while self.completed_index.unwrap() < event_index {
            let started_index = self.history[self.started_index + 1..]
                .iter()
                .position(|event| event.event_type == EventType::OrchestratorStarted)
                .map(|pos| pos + self.started_index + 1);

            if started_index.is_none() {
                return;
            }

            self.started_index = started_index.unwrap();
            self.completed_index = self.history[self.started_index..]
                .iter()
                .position(|event| event.event_type == EventType::OrchestratorCompleted)
                .map(|pos| pos + self.started_index);

            self.result.actions.push(Vec::new());

            if self.completed_index.is_none() {
                return;
            }
        }
    }
}
