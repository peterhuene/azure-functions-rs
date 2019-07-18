use crate::{
    durable::OrchestrationState,
    rpc::{typed_data::Data, TypedData},
};
use serde_json::from_str;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

/// Represents the Durable Functions orchestration context binding.
///
/// The following binding attributes are supported:
///
/// | Name            | Description                                                           |
/// |-----------------|-----------------------------------------------------------------------|
/// | `name`          | The name of the parameter being bound.                                |
/// | `orchestration` | The name of the orchestration.  Defaults to the name of the function. |
///
/// # Examples
///
/// TODO: IMPLEMENT
#[derive(Debug)]
pub struct DurableOrchestrationContext {
    state: Rc<RefCell<OrchestrationState>>,
}

impl DurableOrchestrationContext {
    #[doc(hidden)]
    pub fn new(data: TypedData, _metadata: HashMap<String, TypedData>) -> Self {
        DurableOrchestrationContext {
            state: Rc::new(RefCell::new(match &data.data {
                Some(Data::String(s)) => {
                    from_str(s).expect("failed to parse orchestration context data")
                }
                _ => panic!("expected JSON data for orchestration context data"),
            })),
        }
    }

    #[doc(hidden)]
    pub fn get_state(&self) -> Rc<RefCell<OrchestrationState>> {
        self.state.clone()
    }
}

/*
{
   "history":[
      {
         "EventType":12,
         "EventId":-1,
         "IsPlayed":false,
         "Timestamp":"2019-07-18T06:22:27.016757Z"
      },
      {
         "OrchestrationInstance":{
            "InstanceId":"49497890673e4a75ab380e7a956c607b",
            "ExecutionId":"5d2025984bef476bbaacefaa499a4f5f"
         },
         "EventType":0,
         "ParentInstance":null,
         "Name":"HelloWorld",
         "Version":"",
         "Input":"{}",
         "Tags":null,
         "EventId":-1,
         "IsPlayed":false,
         "Timestamp":"2019-07-18T06:22:26.626966Z"
      }
   ],
   "input":{

   },
   "instanceId":"49497890673e4a75ab380e7a956c607b",
   "isReplaying":false,
   "parentInstanceId":null
}
*/

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::typed_data::Data;

    #[test]
    fn it_constructs() {
        let data = TypedData {
            data: Some(Data::String(r#"{ }"#.to_owned())),
        };

        let _ = DurableOrchestrationContext::new(data, HashMap::new());

        // TODO: implement
    }
}
