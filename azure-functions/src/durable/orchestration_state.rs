use crate::durable::{Action, EventType, HistoryEvent};
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{to_string, Value};
use sha1::Sha1;
use uuid::Uuid;

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
    guid_counter: u32,
}

impl OrchestrationState {
    pub(crate) fn new(history: Vec<HistoryEvent>) -> Self {
        let started_index = history
            .iter()
            .position(|event| event.event_type == EventType::OrchestratorStarted)
            .expect("failed to find orchestrator started event");

        let completed_index = history[started_index..]
            .iter()
            .position(|event| event.event_type == EventType::OrchestratorCompleted)
            .map(|pos| pos + started_index);

        Self {
            history,
            result: ExecutionResult::default(),
            started_index,
            completed_index,
            guid_counter: 0,
        }
    }

    pub(crate) fn is_replaying(&self) -> bool {
        match self.completed_index {
            Some(i) => self.history.len() != (i + 1),
            None => false,
        }
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

    pub(crate) fn set_custom_status(&mut self, value: Value) {
        self.result.custom_status = Some(value);
    }

    pub(crate) fn result(&self) -> String {
        to_string(&self.result).unwrap()
    }

    pub(crate) fn find_start_event(
        &mut self,
        name: &str,
        event_type: EventType,
    ) -> Option<(usize, &mut HistoryEvent)> {
        let index = self.history.iter().position(|event| {
            !event.is_processed
                && event.event_type == event_type
                && event.name.as_ref().map(|n| n.as_ref()) == Some(name)
        })?;

        Some((index, &mut self.history[index]))
    }

    pub(crate) fn find_end_event(
        &mut self,
        start_index: usize,
        completed_type: EventType,
        failed_type: Option<EventType>,
    ) -> Option<(usize, &mut HistoryEvent)> {
        if start_index + 1 >= self.history.len() {
            return None;
        }

        let id = self.history[start_index].event_id;

        let index = self.history[start_index + 1..]
            .iter()
            .position(|event| {
                !event.is_processed
                    && (event.event_type == completed_type
                        || (failed_type.is_some()
                            && event.event_type == *failed_type.as_ref().unwrap()))
                    && event.task_scheduled_id == Some(id)
            })
            .map(|p| p + start_index + 1)?;

        Some((index, &mut self.history[index]))
    }

    pub(crate) fn find_timer_created(&mut self) -> Option<(usize, &mut HistoryEvent)> {
        let index = self
            .history
            .iter()
            .position(|event| !event.is_processed && event.event_type == EventType::TimerCreated)?;

        Some((index, &mut self.history[index]))
    }

    pub(crate) fn find_timer_fired(
        &mut self,
        created_index: usize,
    ) -> Option<(usize, &mut HistoryEvent)> {
        if created_index + 1 >= self.history.len() {
            return None;
        }

        let id = self.history[created_index].event_id;

        let index = self.history[created_index + 1..]
            .iter()
            .position(|event| {
                !event.is_processed
                    && event.event_type == EventType::TimerFired
                    && event.timer_id == Some(id)
            })
            .map(|p| p + created_index + 1)?;

        Some((index, &mut self.history[index]))
    }

    pub(crate) fn find_event_raised(&mut self, name: &str) -> Option<(usize, &mut HistoryEvent)> {
        let index = self.history.iter().position(|event| {
            !event.is_processed
                && event.event_type == EventType::EventRaised
                && event.name.as_ref().map(|n| n.as_ref()) == Some(name)
        })?;

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

    pub(crate) fn new_guid(&mut self, instance_id: &str) -> uuid::Uuid {
        const GUID_NAMESPACE: &str = "9e952958-5e33-4daf-827f-2fa12937b875";

        let mut hasher = Sha1::new();
        hasher.update(
            format!(
                "{}_{}_{}",
                instance_id,
                self.current_time().to_string(),
                self.guid_counter
            )
            .as_bytes(),
        );

        self.guid_counter += 1;

        Uuid::new_v5(
            &Uuid::parse_str(GUID_NAMESPACE).expect("failed to parse namespace GUID"),
            &hasher.digest().bytes(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::durable::tests::create_event;
    use serde_json::json;

    #[test]
    #[should_panic(expected = "failed to find orchestrator started event")]
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

        assert_eq!(
            state.find_start_event("foo", EventType::TaskScheduled),
            None
        );
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

        match state.find_start_event("foo", EventType::TaskScheduled) {
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

        match state.find_start_event("foo", EventType::TaskScheduled) {
            Some((idx, entry)) => {
                assert_eq!(idx, 1);
                assert_eq!(entry.event_type, EventType::TaskScheduled);
                assert_eq!(
                    state.find_end_event(
                        idx,
                        EventType::TaskCompleted,
                        Some(EventType::TaskFailed)
                    ),
                    None
                );
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

        match state.find_start_event("foo", EventType::TaskScheduled) {
            Some((idx, entry)) => {
                assert_eq!(idx, 1);
                assert_eq!(entry.event_type, EventType::TaskScheduled);
                match state.find_end_event(
                    idx,
                    EventType::TaskCompleted,
                    Some(EventType::TaskFailed),
                ) {
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

        match state.find_start_event("foo", EventType::TaskScheduled) {
            Some((idx, entry)) => {
                assert_eq!(idx, 1);
                assert_eq!(entry.event_type, EventType::TaskScheduled);
                match state.find_end_event(
                    idx,
                    EventType::TaskCompleted,
                    Some(EventType::TaskFailed),
                ) {
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
