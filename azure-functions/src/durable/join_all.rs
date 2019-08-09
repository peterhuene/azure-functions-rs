use crate::durable::{OrchestrationFuture, OrchestrationState};
use futures::future::{join_all, FutureExt};
use std::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

/// Future for the `DurableOrchestrationContext::join_all` function.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct JoinAll<F>
where
    F: OrchestrationFuture,
{
    inner: futures::future::JoinAll<F>,
    state: Rc<RefCell<OrchestrationState>>,
    event_index: Option<usize>,
    is_inner: bool,
}

impl<F> JoinAll<F>
where
    F: OrchestrationFuture,
{
    pub(crate) fn new<T>(state: Rc<RefCell<OrchestrationState>>, iter: T) -> Self
    where
        T: IntoIterator<Item = F>,
        F: OrchestrationFuture,
    {
        let inner: Vec<_> = iter
            .into_iter()
            .map(|mut f| {
                f.notify_inner();
                f
            })
            .collect();

        // The event index of a join is the maximum of the sequence, provided all inner futures have event indexes
        let event_index = inner
            .iter()
            .try_fold(None, |i, f| {
                let next = f.event_index();
                if next.is_none() {
                    Err(())
                } else if i < next {
                    Ok(next)
                } else {
                    Ok(i)
                }
            })
            .unwrap_or(None);

        JoinAll {
            inner: join_all(inner),
            state,
            event_index,
            is_inner: false,
        }
    }
}

impl<F> Future for JoinAll<F>
where
    F: OrchestrationFuture,
{
    type Output = Vec<F::Output>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context) -> Poll<Self::Output> {
        let result = self.inner.poll_unpin(context);

        if !self.is_inner {
            if let Poll::Ready(_) = &result {
                self.state.borrow_mut().update(self.event_index.unwrap());
            }
        }

        result
    }
}

impl<F> OrchestrationFuture for JoinAll<F>
where
    F: OrchestrationFuture,
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
        ActionFuture, EventType,
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
        let future1 = ActionFuture::<()>::new(None, state.clone(), None);
        let future2 = ActionFuture::<()>::new(None, state.clone(), None);
        let join = JoinAll::new(state.clone(), vec![future1, future2]);

        assert_eq!(join.event_index(), None);
        assert_eq!(poll(join), Poll::Pending);
    }

    #[test]
    fn it_polls_pending_with_a_result() {
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
                EventType::TaskScheduled,
                1,
                Some("world".to_string()),
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
            create_event(
                EventType::TaskCompleted,
                -1,
                Some("world".to_string()),
                Some(json!("world").to_string()),
                Some(1),
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

        let result1 = Some(from_str(&event.result.as_ref().unwrap()).unwrap());
        let idx1 = Some(idx);

        let (idx, event) = state
            .find_start_event("world", EventType::TaskScheduled)
            .unwrap();
        event.is_processed = true;

        let (idx, event) = state
            .find_end_event(idx, EventType::TaskCompleted, Some(EventType::TaskFailed))
            .unwrap();
        event.is_processed = true;

        let result2 = Some(from_str(&event.result.as_ref().unwrap()).unwrap());
        let idx2 = Some(idx);

        let state = Rc::new(RefCell::new(state));
        let future1 = ActionFuture::new(result1, state.clone(), idx1);
        let future2 = ActionFuture::new(result2, state.clone(), idx2);
        let join = JoinAll::new(state.clone(), vec![future2, future1]);

        assert_eq!(join.event_index(), idx2);
        assert_eq!(
            poll(join),
            Poll::Ready(vec![json!("world"), json!("hello")])
        );
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
            create_event(
                EventType::TaskScheduled,
                1,
                Some("world".to_string()),
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
            create_event(EventType::OrchestratorCompleted, -1, None, None, None),
            create_event(EventType::OrchestratorStarted, -1, None, None, None),
            create_event(
                EventType::TaskCompleted,
                -1,
                Some("world".to_string()),
                Some(json!("world").to_string()),
                Some(1),
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

        let result1 = Some(from_str(&event.result.as_ref().unwrap()).unwrap());
        let idx1 = Some(idx);

        let (idx, event) = state
            .find_start_event("world", EventType::TaskScheduled)
            .unwrap();
        event.is_processed = true;

        let (idx, event) = state
            .find_end_event(idx, EventType::TaskCompleted, Some(EventType::TaskFailed))
            .unwrap();
        event.is_processed = true;

        let result2 = Some(from_str(&event.result.as_ref().unwrap()).unwrap());
        let idx2 = Some(idx);

        let state = Rc::new(RefCell::new(state));
        let future1 = ActionFuture::new(result1, state.clone(), idx1);
        let future2 = ActionFuture::new(result2, state.clone(), idx2);
        let join = JoinAll::new(state.clone(), vec![future2, future1]);

        assert_eq!(join.event_index(), idx2);
        assert_eq!(
            poll(join),
            Poll::Ready(vec![json!("world"), json!("hello")])
        );
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
            create_event(
                EventType::TaskScheduled,
                1,
                Some("world".to_string()),
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
            create_event(EventType::OrchestratorCompleted, -1, None, None, None),
            create_event(EventType::OrchestratorStarted, -1, None, None, None),
            create_event(
                EventType::TaskCompleted,
                -1,
                Some("world".to_string()),
                Some(json!("world").to_string()),
                Some(1),
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

        let result1 = Some(from_str(&event.result.as_ref().unwrap()).unwrap());
        let idx1 = Some(idx);

        let (idx, event) = state
            .find_start_event("world", EventType::TaskScheduled)
            .unwrap();
        event.is_processed = true;

        let (idx, event) = state
            .find_end_event(idx, EventType::TaskCompleted, Some(EventType::TaskFailed))
            .unwrap();
        event.is_processed = true;

        let result2 = Some(from_str(&event.result.as_ref().unwrap()).unwrap());
        let idx2 = Some(idx);

        let state = Rc::new(RefCell::new(state));
        let future1 = ActionFuture::new(result1, state.clone(), idx1);
        let future2 = ActionFuture::new(result2, state.clone(), idx2);
        let mut join = JoinAll::new(state.clone(), vec![future2, future1]);
        join.notify_inner();

        assert_eq!(join.event_index(), idx2);
        assert_eq!(
            poll(join),
            Poll::Ready(vec![json!("world"), json!("hello")])
        );
        assert!(state.borrow().is_replaying());
    }
}
