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
