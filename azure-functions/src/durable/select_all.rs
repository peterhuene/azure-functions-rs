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
