use crate::codegen::{bindings, Function};
use lazy_static::lazy_static;
use semver::Version;
use std::collections::{hash_map::Iter, HashMap};

// Note: package names are expected to be lowercase.
const STORAGE_PACKAGE_NAME: &str = "microsoft.azure.webjobs.extensions.storage";
const STORAGE_PACKAGE_VERSION: &str = "3.0.3";
const EVENT_GRID_PACKAGE_NAME: &str = "microsoft.azure.webjobs.extensions.eventgrid";
const EVENT_GRID_PACKAGE_VERSION: &str = "2.0.0";
const EVENT_HUBS_PACKAGE_NAME: &str = "microsoft.azure.webjobs.extensions.eventhubs";
const EVENT_HUBS_PACKAGE_VERSION: &str = "3.0.3";
const COSMOS_DB_PACKAGE_NAME: &str = "microsoft.azure.webjobs.extensions.cosmosdb";
const COSMOS_DB_PACKAGE_VERSION: &str = "3.0.3";
const SIGNALR_PACKAGE_NAME: &str = "microsoft.azure.webjobs.extensions.signalrservice";
const SIGNALR_PACKAGE_VERSION: &str = "1.0.0";
const SERVICE_BUS_PACKAGE_NAME: &str = "microsoft.azure.webjobs.extensions.servicebus";
const SERVICE_BUS_PACKAGE_VERSION: &str = "3.0.3";
const TWILIO_PACKAGE_NAME: &str = "microsoft.azure.webjobs.extensions.twilio";
const TWILIO_PACKAGE_VERSION: &str = "3.0.0";
const SEND_GRID_PACKAGE_NAME: &str = "microsoft.azure.webjobs.extensions.sendgrid";
const SEND_GRID_PACKAGE_VERSION: &str = "3.0.0";
const DURABLE_TASK_PACKAGE_NAME: &str = "microsoft.azure.webjobs.extensions.durabletask";
const DURABLE_TASK_PACKAGE_VERSION: &str = "1.8.3";

lazy_static! {
    // This comes from https://github.com/Azure/azure-functions-core-tools/blob/master/src/Azure.Functions.Cli/Common/Constants.cs#L63
    static ref BINDING_EXTENSIONS: HashMap<&'static str, (&'static str, &'static str)> = {
        let mut map = HashMap::new();
        map.insert(
            bindings::BlobTrigger::binding_type(),
            (STORAGE_PACKAGE_NAME, STORAGE_PACKAGE_VERSION),
        );
        map.insert(
            bindings::Blob::binding_type(),
            (STORAGE_PACKAGE_NAME, STORAGE_PACKAGE_VERSION),
        );
        map.insert(
            bindings::Queue::binding_type(),
            (STORAGE_PACKAGE_NAME, STORAGE_PACKAGE_VERSION),
        );
        map.insert(
            bindings::QueueTrigger::binding_type(),
            (STORAGE_PACKAGE_NAME, STORAGE_PACKAGE_VERSION),
        );
        map.insert(
            bindings::Table::binding_type(),
            (STORAGE_PACKAGE_NAME, STORAGE_PACKAGE_VERSION),
        );
        map.insert(
            bindings::EventGridTrigger::binding_type(),
            (EVENT_GRID_PACKAGE_NAME, EVENT_GRID_PACKAGE_VERSION),
        );
        map.insert(
            bindings::EventHubTrigger::binding_type(),
            (EVENT_HUBS_PACKAGE_NAME, EVENT_HUBS_PACKAGE_VERSION),
        );
        map.insert(
            bindings::EventHub::binding_type(),
            (EVENT_HUBS_PACKAGE_NAME, EVENT_HUBS_PACKAGE_VERSION),
        );
        map.insert(
            bindings::CosmosDbTrigger::binding_type(),
            (COSMOS_DB_PACKAGE_NAME, COSMOS_DB_PACKAGE_VERSION),
        );
        map.insert(
            bindings::CosmosDb::binding_type(),
            (COSMOS_DB_PACKAGE_NAME, COSMOS_DB_PACKAGE_VERSION),
        );
        map.insert(
            bindings::SignalRConnectionInfo::binding_type(),
            (SIGNALR_PACKAGE_NAME, SIGNALR_PACKAGE_VERSION),
        );
        map.insert(
            bindings::SignalR::binding_type(),
            (SIGNALR_PACKAGE_NAME, SIGNALR_PACKAGE_VERSION),
        );
        map.insert(
            bindings::ServiceBusTrigger::binding_type(),
            (SERVICE_BUS_PACKAGE_NAME, SERVICE_BUS_PACKAGE_VERSION),
        );
        map.insert(
            bindings::ServiceBus::binding_type(),
            (SERVICE_BUS_PACKAGE_NAME, SERVICE_BUS_PACKAGE_VERSION),
        );
        map.insert(
            bindings::TwilioSms::binding_type(),
            (TWILIO_PACKAGE_NAME, TWILIO_PACKAGE_VERSION),
        );
        map.insert(
            bindings::SendGrid::binding_type(),
            (SEND_GRID_PACKAGE_NAME, SEND_GRID_PACKAGE_VERSION),
        );
        map.insert(
            bindings::OrchestrationClient::binding_type(),
            (DURABLE_TASK_PACKAGE_NAME, DURABLE_TASK_PACKAGE_VERSION),
        );
        map.insert(
            bindings::OrchestrationTrigger::binding_type(),
            (DURABLE_TASK_PACKAGE_NAME, DURABLE_TASK_PACKAGE_VERSION),
        );
        map.insert(
            bindings::ActivityTrigger::binding_type(),
            (DURABLE_TASK_PACKAGE_NAME, DURABLE_TASK_PACKAGE_VERSION),
        );
        map
    };
}

pub struct Registry<'a> {
    functions: HashMap<String, &'a Function>,
    registered: HashMap<String, &'a Function>,
}

impl<'a> Registry<'a> {
    pub fn new(functions: &[&'a Function]) -> Registry<'a> {
        Registry {
            functions: functions
                .iter()
                .by_ref()
                .fold(HashMap::new(), |mut map, func| {
                    if map.insert(func.name.clone().into_owned(), func).is_some() {
                        panic!("Azure Function '{}' has already been registered; ensure all functions have unique names.", func.name);
                    }
                    map
                }),
            registered: HashMap::new(),
        }
    }

    pub fn register(&mut self, id: &str, name: &str) -> bool {
        match self.functions.get(name) {
            Some(info) => {
                self.registered.insert(id.to_owned(), info);
                true
            }
            None => false,
        }
    }

    pub fn get(&self, id: &str) -> Option<&'a Function> {
        self.registered.get(id).cloned()
    }

    pub fn iter(&self) -> Iter<String, &'a Function> {
        self.functions.iter()
    }

    pub fn build_extensions_map(&self, extensions: &[(&str, &str)]) -> HashMap<String, String> {
        let mut map = HashMap::new();

        for function in self.functions.iter() {
            for binding in function.1.bindings.iter() {
                if let Some(t) = binding.binding_type() {
                    if let Some(extension) = BINDING_EXTENSIONS.get(t) {
                        Self::insert_extension(&mut map, extension.0, extension.1);
                    }
                }
            }
        }

        for extension in extensions {
            Self::insert_extension(&mut map, &extension.0.to_lowercase(), extension.1);
        }

        map
    }

    fn insert_extension(map: &mut HashMap<String, String>, name: &str, version: &str) {
        match map.get_mut(name) {
            Some(current) => {
                if Version::parse(version) > Version::parse(current) {
                    *current = version.to_owned();
                }
            }
            None => {
                map.insert(name.to_owned(), version.to_owned());
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::bindings::{Binding, Direction};
    use std::borrow::Cow;

    #[test]
    fn it_creates_an_emptry_registry_from_an_empty_slice() {
        let registry = Registry::new(&[]);
        assert_eq!(registry.iter().count(), 0);
    }

    #[test]
    fn it_creates_a_registry_from_a_list_of_functions() {
        let registry = Registry::new(&[
            &Function {
                name: Cow::Borrowed("function1"),
                disabled: false,
                bindings: Cow::Borrowed(&[]),
                invoker: None,
                manifest_dir: None,
                file: None,
            },
            &Function {
                name: Cow::Borrowed("function2"),
                disabled: false,
                bindings: Cow::Borrowed(&[]),
                invoker: None,
                manifest_dir: None,
                file: None,
            },
            &Function {
                name: Cow::Borrowed("function3"),
                disabled: false,
                bindings: Cow::Borrowed(&[]),
                invoker: None,
                manifest_dir: None,
                file: None,
            },
        ]);
        assert_eq!(registry.iter().count(), 3);
        assert!(registry
            .iter()
            .all(|(k, _)| *k == "function1" || *k == "function2" || *k == "function3"));
    }

    #[test]
    fn it_registers_a_function() {
        let mut registry = Registry::new(&[&Function {
            name: Cow::Borrowed("function1"),
            disabled: false,
            bindings: Cow::Borrowed(&[]),
            invoker: None,
            manifest_dir: None,
            file: None,
        }]);
        assert_eq!(registry.iter().count(), 1);

        let p1 = *registry.iter().nth(0).unwrap().1;

        assert!(registry.register("id", "function1"));

        let p2 = registry.get("id").unwrap();
        assert_eq!(p1 as *const _, p2 as *const _);
    }

    #[test]
    fn it_returns_false_if_function_is_not_present() {
        let mut registry = Registry::new(&[&Function {
            name: Cow::Borrowed("function1"),
            disabled: false,
            bindings: Cow::Borrowed(&[]),
            invoker: None,
            manifest_dir: None,
            file: None,
        }]);
        assert_eq!(registry.iter().count(), 1);

        assert_eq!(registry.register("id", "not_present"), false);
    }

    #[test]
    fn it_builds_an_extensions_map() {
        let registry = Registry::new(&[&Function {
            name: Cow::Borrowed("function1"),
            disabled: false,
            bindings: Cow::Borrowed(&[
                Binding::Http(bindings::Http {
                    name: Cow::Borrowed("binding1"),
                }),
                Binding::Queue(bindings::Queue {
                    name: Cow::Borrowed("binding2"),
                    queue_name: Cow::Borrowed("some_queue"),
                    connection: None,
                }),
                Binding::Blob(bindings::Blob {
                    name: Cow::Borrowed("binding3"),
                    path: Cow::Borrowed("some_path"),
                    connection: None,
                    direction: Direction::Out,
                }),
            ]),
            invoker: None,
            manifest_dir: None,
            file: None,
        }]);

        let map = registry.build_extensions_map(&[]);
        assert_eq!(map.len(), 1);
        assert_eq!(
            map.get(STORAGE_PACKAGE_NAME),
            Some(&STORAGE_PACKAGE_VERSION.to_owned())
        );
    }

    #[test]
    fn it_uses_the_latest_extension_version() {
        let registry = Registry::new(&[&Function {
            name: Cow::Borrowed("function"),
            disabled: false,
            bindings: Cow::Borrowed(&[Binding::Queue(bindings::Queue {
                name: Cow::Borrowed("binding"),
                queue_name: Cow::Borrowed("some_queue"),
                connection: None,
            })]),
            invoker: None,
            manifest_dir: None,
            file: None,
        }]);

        let map =
            registry.build_extensions_map(&[(&STORAGE_PACKAGE_NAME.to_uppercase(), "1000.0.0")]);
        assert_eq!(map.len(), 1);
        assert_eq!(map.get(STORAGE_PACKAGE_NAME), Some(&"1000.0.0".to_owned()));
    }

    #[test]
    fn it_builds_an_empty_extensions_map() {
        let registry = Registry::new(&[&Function {
            name: Cow::Borrowed("function1"),
            disabled: false,
            bindings: Cow::Borrowed(&[Binding::Http(bindings::Http {
                name: Cow::Borrowed("binding1"),
            })]),
            invoker: None,
            manifest_dir: None,
            file: None,
        }]);
        assert_eq!(registry.build_extensions_map(&[]).len(), 0);
    }
}
