use crate::durable::{EntityId, EntityState, Signal};
use crate::rpc::{typed_data::Data, TypedData};
use crate::util::nested_json;
use serde::Deserialize;
use serde_json::{from_str, Value};
use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    rc::Rc,
};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Operation {
    name: String,
    #[serde(default)]
    signal: bool,
    #[serde(with = "nested_json", default)]
    input: Value,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct BindingData {
    #[serde(rename = "self")]
    id: EntityId,
    exists: bool,
    #[serde(with = "nested_json")]
    state: Option<Value>,
    batch: Vec<Operation>,
}

/// Represents the Durable Functions entity context binding.
///
/// The following binding attributes are supported:
///
/// | Name       | Description                                                      |
/// |------------|------------------------------------------------------------------|
/// | `name`     | The name of the parameter being bound.                           |
///
/// # Examples
///
/// TODO: implement
#[derive(Clone)]
pub struct DurableEntityContext {
    id: Rc<EntityId>,
    state: Rc<RefCell<EntityState>>,
    batch: Rc<Vec<Operation>>,
    index: usize,
}

impl DurableEntityContext {
    #[doc(hidden)]
    pub fn new(data: TypedData, _metadata: HashMap<String, TypedData>) -> Self {
        match &data.data {
            Some(Data::String(s)) => {
                let data: BindingData = from_str(s).expect("failed to parse entity context data");

                Self {
                    id: Rc::new(data.id),
                    state: Rc::new(RefCell::new(EntityState {
                        exists: data.exists,
                        value: data.state,
                        results: vec![Default::default(); data.batch.len()],
                        ..Default::default()
                    })),
                    batch: Rc::new(data.batch),
                    index: 0,
                }
            }
            _ => panic!("expected JSON data for entity context data"),
        }
    }

    /// The name (class) of the entity.
    pub fn name(&self) -> &str {
        &self.id.name
    }

    /// The unique key of the entity.
    ///
    /// Uniquely identifies an entity among all entities of the same name.
    pub fn key(&self) -> &str {
        &self.id.key
    }

    /// Gets the state of the entity.
    pub fn state(&self) -> Option<Ref<Value>> {
        let state = self.state.borrow();

        state.value.as_ref()?;

        Some(Ref::map(state, |s| s.value.as_ref().unwrap()))
    }

    /// Gets the name of the operation that was called.
    pub fn operation_name(&self) -> &str {
        &self.batch[self.index].name
    }

    /// Gets the input for this operation.
    pub fn input(&self) -> &Value {
        &self.batch[self.index].input
    }

    /// Deletes the state of the entity.
    pub fn delete_state(&self) {
        let mut state = self.state.borrow_mut();
        state.exists = false;
        state.value = None;
    }

    /// Sets the return value for the caller of this operation.
    pub fn set_return<T>(&self, value: T)
    where
        T: Into<Value>,
    {
        let mut state = self.state.borrow_mut();
        let result = &mut state.results[self.index];
        result.is_error = false;
        result.result = value.into();
    }

    /// Sets the current state of this entity.
    pub fn set_state<T>(&self, value: T)
    where
        T: Into<Value>,
    {
        self.state.borrow_mut().value = Some(value.into());
    }

    /// Signals an entity to perform an operation, without waiting for a response.
    ///
    /// Any result or exception is ignored (fire and forget).
    pub fn signal_entity<T>(&self, name: &str, key: &str, operation: &str, input: T)
    where
        T: Into<Value>,
    {
        let mut state = self.state.borrow_mut();
        state.signals.push(Signal {
            target: EntityId {
                name: name.into(),
                key: key.into(),
            },
            name: operation.into(),
            input: input.into(),
        });
    }

    #[doc(hidden)]
    pub(crate) fn set_index(&mut self, index: usize) {
        self.index = index;
    }

    #[doc(hidden)]
    pub(crate) fn count(&self) -> usize {
        self.batch.len()
    }

    #[doc(hidden)]
    pub(crate) fn internal_state(&self) -> Rc<RefCell<EntityState>> {
        self.state.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::typed_data::Data;

    // #[test]
    // fn it_constructs() {
    //     let data = TypedData {
    //         data: Some(Data::String("bar".to_string())),
    //     };

    //     let mut metadata = HashMap::new();
    //     metadata.insert(
    //         INSTANCE_ID_KEY.to_string(),
    //         TypedData {
    //             data: Some(Data::String("foo".to_string())),
    //         },
    //     );

    //     let context = DurableEntityContext::new(data, metadata);
    // }
}
