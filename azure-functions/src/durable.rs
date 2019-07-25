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
mod history;
mod orchestration_output;

pub use self::actions::*;
pub use self::activity_output::*;
pub use self::history::*;
pub use self::orchestration_output::*;

#[doc(hidden)]
#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionResult {
    is_done: bool,
    actions: Vec<Vec<Action>>,
    output: Option<Value>,
    custom_status: Option<Value>,
    error: Option<String>,
}

impl ExecutionResult {
    pub(crate) fn notify_new_execution(&mut self) {
        self.actions.push(Vec::new());
    }

    pub(crate) fn push_action(&mut self, action: Action) {
        if self.actions.is_empty() {
            self.actions.push(Vec::new());
        }

        self.actions.last_mut().unwrap().push(action);
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
    result: Rc<RefCell<ExecutionResult>>,
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
            let mut result = result.borrow_mut();
            result.output = Some(output.into_value());
            result.is_done = true;
        }
        Poll::Pending => {}
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
