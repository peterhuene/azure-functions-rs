use crate::durable::{OrchestrationFuture, OrchestrationState};
use std::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

/// Future returned by various `DurableOrchestrationContext` functions.
pub struct ActionFuture<T> {
    result: Option<T>,
    state: Rc<RefCell<OrchestrationState>>,
    event_index: Option<usize>,
    is_inner: bool,
}

impl<T> ActionFuture<T> {
    pub(crate) fn new(
        result: Option<T>,
        state: Rc<RefCell<OrchestrationState>>,
        event_index: Option<usize>,
    ) -> Self {
        assert!(
            (result.is_none() && event_index.is_none())
                || (result.is_some() && event_index.is_some())
        );

        ActionFuture {
            result,
            state,
            event_index,
            is_inner: false,
        }
    }
}

impl<T> Future for ActionFuture<T>
where
    T: Unpin,
{
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, _context: &mut Context) -> Poll<T> {
        if let Some(v) = self.result.take() {
            if !self.is_inner {
                self.state.borrow_mut().update(self.event_index.unwrap());
            }
            return Poll::Ready(v);
        }

        Poll::Pending
    }
}

impl<T> OrchestrationFuture for ActionFuture<T>
where
    T: Unpin,
{
    fn notify_inner(&mut self) {
        self.is_inner = true;
    }

    fn event_index(&self) -> Option<usize> {
        self.event_index
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::durable::{
        tests::{create_event, poll},
        EventType,
    };
    use serde_json::{from_str, json};
    use std::task::Poll;

    #[test]
    fn it_polls_pending_without_a_result() {
        let history = vec![create_event(
            EventType::OrchestratorStarted,
            -1,
            None,
            None,
            None,
        )];

        let state = Rc::new(RefCell::new(OrchestrationState::new(history)));
        let future = ActionFuture::<()>::new(None, state, None);

        assert_eq!(poll(future), Poll::Pending);
    }

    #[test]
    fn it_polls_ready_given_a_result() {
        let history = vec![
            create_event(EventType::OrchestratorStarted, -1, None, None, None),
            create_event(
                EventType::TaskScheduled,
                0,
                Some("hello".to_string()),
                None,
                None,
            ),
            create_event(
                EventType::TaskCompleted,
                -1,
                Some("hello".to_string()),
                Some(json!("hello").to_string()),
                Some(0),
            ),
        ];

        let mut state = OrchestrationState::new(history);
        let (idx, event) = state
            .find_start_event("hello", EventType::TaskScheduled)
            .unwrap();
        event.is_processed = true;

        let (idx, event) = state
            .find_end_event(idx, EventType::TaskCompleted, Some(EventType::TaskFailed))
            .unwrap();
        event.is_processed = true;

        let result = Some(from_str(&event.result.as_ref().unwrap()).unwrap());
        let state = Rc::new(RefCell::new(state));
        let future = ActionFuture::new(result, state, Some(idx));

        assert_eq!(future.event_index(), Some(idx));
        assert_eq!(poll(future), Poll::Ready(json!("hello")));
    }

    #[test]
    fn it_updates_state() {
        let history = vec![
            create_event(EventType::OrchestratorStarted, -1, None, None, None),
            create_event(
                EventType::TaskScheduled,
                0,
                Some("hello".to_string()),
                None,
                None,
            ),
            create_event(EventType::OrchestratorCompleted, -1, None, None, None),
            create_event(EventType::OrchestratorStarted, -1, None, None, None),
            create_event(
                EventType::TaskCompleted,
                -1,
                Some("hello".to_string()),
                Some(json!("hello").to_string()),
                Some(0),
            ),
        ];

        let mut state = OrchestrationState::new(history);
        assert!(state.is_replaying());

        let (idx, event) = state
            .find_start_event("hello", EventType::TaskScheduled)
            .unwrap();
        event.is_processed = true;

        let (idx, event) = state
            .find_end_event(idx, EventType::TaskCompleted, Some(EventType::TaskFailed))
            .unwrap();
        event.is_processed = true;

        let result = Some(from_str(&event.result.as_ref().unwrap()).unwrap());
        let state = Rc::new(RefCell::new(state));
        let future = ActionFuture::new(result, state.clone(), Some(idx));

        assert_eq!(future.event_index(), Some(idx));
        assert_eq!(poll(future), Poll::Ready(json!("hello")));
        assert!(!state.borrow().is_replaying());
    }

    #[test]
    fn it_does_not_update_state_when_an_inner_future() {
        let history = vec![
            create_event(EventType::OrchestratorStarted, -1, None, None, None),
            create_event(
                EventType::TaskScheduled,
                0,
                Some("hello".to_string()),
                None,
                None,
            ),
            create_event(EventType::OrchestratorCompleted, -1, None, None, None),
            create_event(EventType::OrchestratorStarted, -1, None, None, None),
            create_event(
                EventType::TaskCompleted,
                -1,
                Some("hello".to_string()),
                Some(json!("hello").to_string()),
                Some(0),
            ),
        ];

        let mut state = OrchestrationState::new(history);
        assert!(state.is_replaying());

        let (idx, event) = state
            .find_start_event("hello", EventType::TaskScheduled)
            .unwrap();
        event.is_processed = true;

        let (idx, event) = state
            .find_end_event(idx, EventType::TaskCompleted, Some(EventType::TaskFailed))
            .unwrap();
        event.is_processed = true;

        let result = Some(from_str(&event.result.as_ref().unwrap()).unwrap());
        let state = Rc::new(RefCell::new(state));
        let mut future = ActionFuture::new(result, state.clone(), Some(idx));
        future.notify_inner();

        assert_eq!(future.event_index(), Some(idx));
        assert_eq!(poll(future), Poll::Ready(json!("hello")));
        assert!(state.borrow().is_replaying());
    }
}
