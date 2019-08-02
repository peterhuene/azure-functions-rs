use crate::durable::{OrchestrationFuture, OrchestrationState};
use std::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

pub(crate) struct ActionFuture<T> {
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
