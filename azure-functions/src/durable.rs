//! Module for Durable Functions types.
use crate::rpc::{
    status_result::Status, typed_data::Data, InvocationResponse, StatusResult, TypedData,
};
use serde_json::Value;
use std::{
    cell::RefCell,
    future::Future,
    ptr::null,
    rc::Rc,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

mod action_future;
mod actions;
mod activity_output;
mod history;
mod join_all;
mod orchestration_output;
mod orchestration_state;
mod select_all;

pub use self::action_future::*;
pub use self::actions::*;
pub use self::activity_output::*;
pub(crate) use self::history::*;
pub use self::join_all::*;
pub use self::orchestration_output::*;
pub use self::orchestration_state::*;
pub use self::select_all::*;

/// Represents a Future returned by the orchestration context.
pub trait OrchestrationFuture: Future {
    #[doc(hidden)]
    fn notify_inner(&mut self);

    #[doc(hidden)]
    fn event_index(&self) -> Option<usize>;
}

unsafe fn waker_clone(_: *const ()) -> RawWaker {
    panic!("orchestration functions cannot perform asynchronous operations");
}

unsafe fn waker_wake(_: *const ()) {
    panic!("orchestration functions cannot perform asynchronous operations");
}

unsafe fn waker_wake_by_ref(_: *const ()) {
    panic!("orchestration functions cannot perform asynchronous operations");
}

unsafe fn waker_drop(_: *const ()) {}

#[doc(hidden)]
pub trait IntoValue {
    fn into_value(self) -> Value;
}

impl IntoValue for () {
    fn into_value(self) -> Value {
        Value::Null
    }
}

/// The entrypoint for orchestration functions.
#[doc(hidden)]
pub fn orchestrate<T>(
    id: String,
    func: impl Future<Output = T>,
    state: Rc<RefCell<OrchestrationState>>,
) -> InvocationResponse
where
    T: IntoValue,
{
    let waker = unsafe {
        Waker::from_raw(RawWaker::new(
            null(),
            &RawWakerVTable::new(waker_clone, waker_wake, waker_wake_by_ref, waker_drop),
        ))
    };

    match Future::poll(Box::pin(func).as_mut(), &mut Context::from_waker(&waker)) {
        Poll::Ready(output) => {
            state.borrow_mut().set_output(output.into_value());
        }
        Poll::Pending => {}
    };

    InvocationResponse {
        invocation_id: id,
        return_value: Some(TypedData {
            data: Some(Data::Json(state.borrow().result())),
        }),
        result: Some(StatusResult {
            status: Status::Success as i32,
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::durable::{EventType, HistoryEvent};
    use chrono::Utc;
    use futures::future::FutureExt;
    use std::{
        future::Future,
        ptr::null,
        task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
    };

    pub(crate) fn poll<F, T>(mut future: F) -> Poll<T>
    where
        F: Future<Output = T> + Unpin,
    {
        let waker = unsafe {
            Waker::from_raw(RawWaker::new(
                null(),
                &RawWakerVTable::new(waker_clone, waker_wake, waker_wake_by_ref, waker_drop),
            ))
        };

        future.poll_unpin(&mut Context::from_waker(&waker))
    }

    static mut TIMESTAMP_COUNTER: i64 = 0;

    pub(crate) fn create_event(
        event_type: EventType,
        event_id: i32,
        name: Option<String>,
        result: Option<String>,
        task_scheduled_id: Option<i32>,
    ) -> HistoryEvent {
        unsafe {
            TIMESTAMP_COUNTER += 1;

            let offset = chrono::Duration::nanoseconds(TIMESTAMP_COUNTER);

            HistoryEvent {
                event_type,
                event_id,
                is_played: true,
                timestamp: Utc::now() + offset,
                is_processed: false,
                name,
                input: None,
                result,
                task_scheduled_id,
                instance_id: None,
                reason: None,
                details: None,
                fire_at: None,
                timer_id: None,
            }
        }
    }
}
