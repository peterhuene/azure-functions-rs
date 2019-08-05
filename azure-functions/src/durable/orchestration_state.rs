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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::durable::tests::create_event;
    use serde_json::json;

    #[test]
    #[should_panic(expected = "failed to find orchestration started event")]
    fn it_requires_an_orchestration_start_event() {
        OrchestrationState::new(Vec::new());
    }

    #[test]
    fn it_constructs() {
        let history = vec![create_event(
            EventType::OrchestratorStarted,
            -1,
            None,
            None,
            None,
        )];

        let timestamp = history[0].timestamp;

        let state = OrchestrationState::new(history);

        assert_eq!(state.current_time(), timestamp);
        assert_eq!(state.is_replaying(), false);
    }

    #[test]
    fn it_pushes_an_action() {
        let history = vec![create_event(
            EventType::OrchestratorStarted,
            -1,
            None,
            None,
            None,
        )];

        let mut state = OrchestrationState::new(history);

        let action = Action::CallActivity {
            function_name: "test".to_string(),
            input: json!("hello"),
        };

        state.push_action(action.clone());

        assert_eq!(state.result.actions.len(), 1);
        assert_eq!(state.result.actions[0].len(), 1);
        assert_eq!(state.result.actions[0][0], action);
    }

    #[test]
    fn it_sets_done_with_output() {
        let history = vec![create_event(
            EventType::OrchestratorStarted,
            -1,
            None,
            None,
            None,
        )];

        let mut state = OrchestrationState::new(history);

        state.set_output(json!(42));

        assert!(state.result.is_done);
        assert_eq!(state.result.output.as_ref().unwrap(), &json!(42));
    }

    #[test]
    fn it_returns_a_json_result() {
        let history = vec![create_event(
            EventType::OrchestratorStarted,
            -1,
            None,
            None,
            None,
        )];

        let mut state = OrchestrationState::new(history);

        state.push_action(Action::CallActivity {
            function_name: "test".to_string(),
            input: json!("hello"),
        });

        state.set_output(json!("hello"));

        assert_eq!(
            state.result(),
            r#"{"isDone":true,"actions":[[{"actionType":"callActivity","functionName":"test","input":"hello"}]],"output":"hello","customStatus":null,"error":null}"#
        );
    }

    #[test]
    fn it_returns_none_if_scheduled_activity_is_not_in_history() {
        let history = vec![create_event(
            EventType::OrchestratorStarted,
            -1,
            None,
            None,
            None,
        )];

        let mut state = OrchestrationState::new(history);

        assert_eq!(state.find_scheduled_task("foo"), None);
    }

    #[test]
    fn it_returns_some_if_scheduled_activity_is_in_history() {
        let history = vec![
            create_event(EventType::OrchestratorStarted, -1, None, None, None),
            create_event(
                EventType::TaskScheduled,
                0,
                Some("foo".to_string()),
                None,
                None,
            ),
        ];

        let mut state = OrchestrationState::new(history);

        match state.find_scheduled_task("foo") {
            Some((idx, entry)) => {
                assert_eq!(idx, 1);
                assert_eq!(entry.event_type, EventType::TaskScheduled);
            }
            None => assert!(false),
        }
    }

    #[test]
    fn it_returns_none_if_finished_activity_is_not_in_history() {
        let history = vec![
            create_event(EventType::OrchestratorStarted, -1, None, None, None),
            create_event(
                EventType::TaskScheduled,
                0,
                Some("foo".to_string()),
                None,
                None,
            ),
        ];

        let mut state = OrchestrationState::new(history);

        match state.find_scheduled_task("foo") {
            Some((idx, entry)) => {
                assert_eq!(idx, 1);
                assert_eq!(entry.event_type, EventType::TaskScheduled);
                assert_eq!(state.find_finished_task(idx), None);
            }
            None => assert!(false),
        }
    }

    #[test]
    fn it_returns_some_if_completed_activity_is_in_history() {
        let history = vec![
            create_event(EventType::OrchestratorStarted, -1, None, None, None),
            create_event(
                EventType::TaskScheduled,
                0,
                Some("foo".to_string()),
                None,
                None,
            ),
            create_event(
                EventType::TaskCompleted,
                -1,
                Some("foo".to_string()),
                Some(json!("bar").to_string()),
                Some(0),
            ),
        ];

        let mut state = OrchestrationState::new(history);

        match state.find_scheduled_task("foo") {
            Some((idx, entry)) => {
                assert_eq!(idx, 1);
                assert_eq!(entry.event_type, EventType::TaskScheduled);
                match state.find_finished_task(idx) {
                    Some((idx, entry)) => {
                        assert_eq!(idx, 2);
                        assert_eq!(entry.event_type, EventType::TaskCompleted);
                        assert_eq!(entry.result, Some(json!("bar").to_string()));
                    }
                    None => assert!(false),
                }
            }
            None => assert!(false),
        }
    }

    #[test]
    fn it_returns_some_if_failed_activity_is_in_history() {
        let history = vec![
            create_event(EventType::OrchestratorStarted, -1, None, None, None),
            create_event(
                EventType::TaskScheduled,
                0,
                Some("foo".to_string()),
                None,
                None,
            ),
            create_event(
                EventType::TaskFailed,
                -1,
                Some("foo".to_string()),
                None,
                Some(0),
            ),
        ];

        let mut state = OrchestrationState::new(history);

        match state.find_scheduled_task("foo") {
            Some((idx, entry)) => {
                assert_eq!(idx, 1);
                assert_eq!(entry.event_type, EventType::TaskScheduled);
                match state.find_finished_task(idx) {
                    Some((idx, entry)) => {
                        assert_eq!(idx, 2);
                        assert_eq!(entry.event_type, EventType::TaskFailed);
                    }
                    None => assert!(false),
                }
            }
            None => assert!(false),
        }
    }

    #[test]
    fn it_does_not_update_state_if_there_is_no_completed_event() {
        let history = vec![
            create_event(EventType::OrchestratorStarted, -1, None, None, None),
            create_event(
                EventType::TaskScheduled,
                0,
                Some("foo".to_string()),
                None,
                None,
            ),
            create_event(
                EventType::TaskFailed,
                -1,
                Some("foo".to_string()),
                None,
                Some(0),
            ),
        ];

        let mut state = OrchestrationState::new(history);
        assert!(!state.is_replaying());

        let current_time = state.current_time();

        state.update(2);

        assert_eq!(state.current_time(), current_time);
        assert!(!state.is_replaying());
    }

    #[test]
    fn it_does_not_update_state_if_index_is_less_than_end() {
        let history = vec![
            create_event(EventType::OrchestratorStarted, -1, None, None, None),
            create_event(
                EventType::TaskScheduled,
                0,
                Some("foo".to_string()),
                None,
                None,
            ),
            create_event(
                EventType::TaskFailed,
                -1,
                Some("foo".to_string()),
                None,
                Some(0),
            ),
            create_event(EventType::OrchestratorCompleted, -1, None, None, None),
            create_event(EventType::OrchestratorStarted, -1, None, None, None),
        ];

        let mut state = OrchestrationState::new(history);
        assert!(state.is_replaying());

        let current_time = state.current_time();

        state.update(2);

        assert_eq!(state.current_time(), current_time);
        assert!(state.is_replaying());
    }

    #[test]
    fn it_updates_when_the_index_is_greater_with_end() {
        let history = vec![
            create_event(EventType::OrchestratorStarted, -1, None, None, None),
            create_event(
                EventType::TaskScheduled,
                0,
                Some("foo".to_string()),
                None,
                None,
            ),
            create_event(EventType::OrchestratorCompleted, -1, None, None, None),
            create_event(EventType::OrchestratorStarted, -1, None, None, None),
            create_event(
                EventType::TaskFailed,
                -1,
                Some("foo".to_string()),
                None,
                Some(0),
            ),
            create_event(EventType::OrchestratorCompleted, -1, None, None, None),
            create_event(EventType::OrchestratorStarted, -1, None, None, None),
        ];

        let mut state = OrchestrationState::new(history);
        assert!(state.is_replaying());

        let current_time = state.current_time();

        state.update(4);

        assert_ne!(state.current_time(), current_time);
        assert!(state.is_replaying());
    }

    #[test]
    fn it_updates_when_the_index_is_greater() {
        let history = vec![
            create_event(EventType::OrchestratorStarted, -1, None, None, None),
            create_event(
                EventType::TaskScheduled,
                0,
                Some("foo".to_string()),
                None,
                None,
            ),
            create_event(EventType::OrchestratorCompleted, -1, None, None, None),
            create_event(EventType::OrchestratorStarted, -1, None, None, None),
            create_event(
                EventType::TaskFailed,
                -1,
                Some("foo".to_string()),
                None,
                Some(0),
            ),
        ];

        let mut state = OrchestrationState::new(history);
        assert!(state.is_replaying());

        let current_time = state.current_time();

        state.update(4);

        assert_ne!(state.current_time(), current_time);
        assert!(!state.is_replaying());
    }
}
