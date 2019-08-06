use crate::durable::{OrchestrationFuture, OrchestrationState};
use futures::future::FutureExt;
use std::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

/// Future for the `DurableOrchestrationContext::select_all` function.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct SelectAll<F> {
    inner: Vec<F>,
    state: Rc<RefCell<OrchestrationState>>,
    event_index: Option<usize>,
    is_inner: bool,
}

impl<F> SelectAll<F>
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

        assert!(!inner.is_empty());

        // The event index of a select is the minimum present index of the sequence
        let event_index = inner.iter().filter_map(|f| f.event_index()).min();

        SelectAll {
            inner,
            state,
            event_index,
            is_inner: false,
        }
    }
}

impl<F> Unpin for SelectAll<F> where F: Unpin {}

impl<F> Future for SelectAll<F>
where
    F: OrchestrationFuture + Unpin,
{
    type Output = (F::Output, usize, Vec<F>);

    fn poll(mut self: Pin<&mut Self>, context: &mut Context) -> Poll<Self::Output> {
        let event_index = self.event_index;
        if event_index.is_none() {
            return Poll::Pending;
        }

        let item = self.inner.iter_mut().enumerate().find_map(|(i, f)| {
            if f.event_index() != event_index {
                return None;
            }
            match f.poll_unpin(context) {
                Poll::Pending => None,
                Poll::Ready(e) => Some((i, e)),
            }
        });

        match item {
            Some((idx, res)) => {
                self.inner.remove(idx);
                let rest = std::mem::replace(&mut self.inner, Vec::new());

                if !self.is_inner {
                    self.state.borrow_mut().update(event_index.unwrap());
                }

                Poll::Ready((res, idx, rest))
            }
            None => Poll::Pending,
        }
    }
}

impl<F> OrchestrationFuture for SelectAll<F>
where
    F: OrchestrationFuture + Unpin,
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
    use serde_json::{from_str, json, Value};
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
        let select = SelectAll::new(state.clone(), vec![future1, future2]);

        assert_eq!(select.event_index(), None);
        match poll(select) {
            Poll::Ready(_) => assert!(false),
            Poll::Pending => {}
        };
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

        let (idx, event) = state.find_scheduled_task("hello").unwrap();
        event.is_processed = true;

        let (idx, event) = state.find_finished_task(idx).unwrap();
        event.is_processed = true;

        let result1 = Some(from_str(&event.result.as_ref().unwrap()).unwrap());
        let idx1 = Some(idx);

        let (idx, event) = state.find_scheduled_task("world").unwrap();
        event.is_processed = true;

        let (idx, event) = state.find_finished_task(idx).unwrap();
        event.is_processed = true;

        let result2 = Some(from_str(&event.result.as_ref().unwrap()).unwrap());
        let idx2 = Some(idx);

        let state = Rc::new(RefCell::new(state));
        let future1 = ActionFuture::new(result1, state.clone(), idx1);
        let future2 = ActionFuture::new(result2, state.clone(), idx2);
        let select = SelectAll::new(state.clone(), vec![future2, future1]);

        assert_eq!(select.event_index(), idx1);
        match poll(select) {
            Poll::Ready((r, i, mut remaining)) => {
                assert_eq!(r, json!("hello"));
                assert_eq!(i, 1);
                assert_eq!(remaining.len(), 1);
                assert_eq!(poll(remaining.pop().unwrap()), Poll::Ready(json!("world")));
            }
            Poll::Pending => assert!(false),
        };
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

        let (idx, event) = state.find_scheduled_task("hello").unwrap();
        event.is_processed = true;

        let (idx, event) = state.find_finished_task(idx).unwrap();
        event.is_processed = true;

        let result1: Option<Value> = Some(from_str(&event.result.as_ref().unwrap()).unwrap());
        let idx1 = Some(idx);

        let (idx, event) = state.find_scheduled_task("world").unwrap();
        event.is_processed = true;

        let (idx, event) = state.find_finished_task(idx).unwrap();
        event.is_processed = true;

        let result2 = Some(from_str(&event.result.as_ref().unwrap()).unwrap());
        let idx2 = Some(idx);

        let state = Rc::new(RefCell::new(state));
        let future1 = ActionFuture::new(result1, state.clone(), idx1);
        let future2 = ActionFuture::new(result2, state.clone(), idx2);
        let select = SelectAll::new(state.clone(), vec![future2, future1]);

        assert_eq!(select.event_index(), idx1);
        match poll(select) {
            Poll::Ready((r, i, remaining)) => {
                assert_eq!(r, json!("hello"));
                assert_eq!(i, 1);
                assert_eq!(remaining.len(), 1);

                match poll(SelectAll::new(state.clone(), remaining)) {
                    Poll::Ready((r, i, remaining)) => {
                        assert_eq!(r, json!("world"));
                        assert_eq!(i, 0);
                        assert!(remaining.is_empty());
                        assert!(!state.borrow().is_replaying());
                    }
                    Poll::Pending => assert!(false),
                };
            }
            Poll::Pending => assert!(false),
        };
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

        let (idx, event) = state.find_scheduled_task("hello").unwrap();
        event.is_processed = true;

        let (idx, event) = state.find_finished_task(idx).unwrap();
        event.is_processed = true;

        let result1: Option<Value> = Some(from_str(&event.result.as_ref().unwrap()).unwrap());
        let idx1 = Some(idx);

        let (idx, event) = state.find_scheduled_task("world").unwrap();
        event.is_processed = true;

        let (idx, event) = state.find_finished_task(idx).unwrap();
        event.is_processed = true;

        let result2 = Some(from_str(&event.result.as_ref().unwrap()).unwrap());
        let idx2 = Some(idx);

        let state = Rc::new(RefCell::new(state));
        let future1 = ActionFuture::new(result1, state.clone(), idx1);
        let future2 = ActionFuture::new(result2, state.clone(), idx2);
        let mut select = SelectAll::new(state.clone(), vec![future2, future1]);
        select.notify_inner();

        assert_eq!(select.event_index(), idx1);
        match poll(select) {
            Poll::Ready((r, i, remaining)) => {
                assert_eq!(r, json!("hello"));
                assert_eq!(i, 1);
                assert_eq!(remaining.len(), 1);

                let mut select = SelectAll::new(state.clone(), remaining);
                select.notify_inner();

                match poll(select) {
                    Poll::Ready((r, i, remaining)) => {
                        assert_eq!(r, json!("world"));
                        assert_eq!(i, 0);
                        assert!(remaining.is_empty());
                        assert!(state.borrow().is_replaying());
                    }
                    Poll::Pending => assert!(false),
                };
            }
            Poll::Pending => assert!(false),
        };
    }
}
