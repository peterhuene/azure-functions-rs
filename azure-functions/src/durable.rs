//! Module for Durable Functions types.
use crate::rpc::{
    status_result::Status, typed_data::Data, InvocationResponse, StatusResult, TypedData,
};
use serde::Serialize;
use serde_json::{to_string, Value};
use std::{
    cell::RefCell,
    future::Future,
    ptr::null,
    rc::Rc,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

mod actions;
mod activity_output;
mod creation_urls;
mod history;
mod management_urls;
mod orchestrator_output;

pub use actions::*;
pub use creation_urls::*;
pub use history::*;
pub use management_urls::*;

pub use self::activity_output::*;
pub use self::creation_urls::*;
pub use self::management_urls::*;
pub use self::orchestrator_output::*;

#[doc(hidden)]
#[derive(Debug, Serialize, Default)]
pub struct ExecutionResult {
    done: bool,
    actions: Vec<Action>,
    output: Option<Value>,
    custom_status: Option<Value>,
    error: Option<String>,
}

impl ExecutionResult {
    fn mark_done(&mut self) {
        self.done = true;
    }

    pub(crate) fn add_action(&mut self, action: Action) {
        self.actions.push(action);
    }
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

/// The entrypoint for orchestration functions.
///
/// The given future is the user function.
#[doc(hidden)]
pub fn orchestrate(
    id: String,
    func: impl Future<Output = ()>,
    result: Rc<RefCell<ExecutionResult>>,
) -> InvocationResponse {
    let waker = unsafe {
        Waker::from_raw(RawWaker::new(
            null(),
            &RawWakerVTable::new(waker_clone, waker_wake, waker_wake_by_ref, waker_drop),
        ))
    };

    match Future::poll(Box::pin(func).as_mut(), &mut Context::from_waker(&waker)) {
        Poll::Ready(_) => {
            // Orchestration has completed and the result is ready, return done with output
            result.as_ref().borrow_mut().mark_done();
        }
        Poll::Pending => {
            // Orchestration has not yet completed
        }
    };

    InvocationResponse {
        invocation_id: id,
        return_value: Some(TypedData {
            data: Some(Data::Json(to_string(&*result.borrow()).unwrap())),
        }),
        result: Some(StatusResult {
            status: Status::Success as i32,
            ..Default::default()
        }),
        ..Default::default()
    }
}
