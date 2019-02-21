use crate::codegen::{bindings, Function};
use std::collections::hash_map::Iter;
use std::collections::HashMap;

const STORAGE_PACKAGE_NAME: &str = "Microsoft.Azure.WebJobs.Extensions.Storage";
const STORAGE_PACKAGE_VERSION: &str = "3.0.3";
const EVENT_GRID_PACKAGE_NAME: &str = "Microsoft.Azure.WebJobs.Extensions.EventGrid";
const EVENT_GRID_PACKAGE_VERSION: &str = "2.0.0";
const EVENT_HUBS_PACKAGE_NAME: &str = "Microsoft.Azure.WebJobs.Extensions.EventHubs";
const EVENT_HUBS_PACKAGE_VERSION: &str = "3.0.2";

lazy_static! {
    // This comes from https://github.com/Azure/azure-functions-core-tools/blob/master/src/Azure.Functions.Cli/Common/Constants.cs#L63
    static ref BINDING_EXTENSIONS: HashMap<&'static str, (&'static str, &'static str)> = {
        let mut map = HashMap::new();
        map.insert(
            bindings::BLOB_TRIGGER_TYPE,
            (STORAGE_PACKAGE_NAME, STORAGE_PACKAGE_VERSION),
        );
        map.insert(
            bindings::BLOB_TYPE,
            (STORAGE_PACKAGE_NAME, STORAGE_PACKAGE_VERSION),
        );
        map.insert(
            bindings::QUEUE_TYPE,
            (STORAGE_PACKAGE_NAME, STORAGE_PACKAGE_VERSION),
        );
        map.insert(
            bindings::QUEUE_TRIGGER_TYPE,
            (STORAGE_PACKAGE_NAME, STORAGE_PACKAGE_VERSION),
        );
        map.insert(
            bindings::TABLE_TYPE,
            (STORAGE_PACKAGE_NAME, STORAGE_PACKAGE_VERSION),
        );
        map.insert(
            bindings::EVENT_GRID_TRIGGER_TYPE,
            (EVENT_GRID_PACKAGE_NAME, EVENT_GRID_PACKAGE_VERSION),
        );
        map.insert(
            bindings::EVENT_HUB_TRIGGER_TYPE,
            (EVENT_HUBS_PACKAGE_NAME, EVENT_HUBS_PACKAGE_VERSION),
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

    pub fn has_binding_extensions(&self) -> bool {
        for function in self.functions.iter() {
            for binding in function.1.bindings.iter() {
                if let Some(t) = binding.binding_type() {
                    if BINDING_EXTENSIONS.get(t).is_some() {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn iter_binding_extensions(&self) -> impl Iterator<Item = (&'static str, &'static str)> {
        let mut set = std::collections::HashSet::new();
        for function in self.functions.iter() {
            for binding in function.1.bindings.iter() {
                if let Some(t) = binding.binding_type() {
                    if let Some(extension) = BINDING_EXTENSIONS.get(t) {
                        set.insert(extension.clone());
                    }
                }
            }
        }
        set.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::{Binding, Direction};
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
                invoker_name: None,
                invoker: None,
                manifest_dir: None,
                file: None,
            },
            &Function {
                name: Cow::Borrowed("function2"),
                disabled: false,
                bindings: Cow::Borrowed(&[]),
                invoker_name: None,
                invoker: None,
                manifest_dir: None,
                file: None,
            },
            &Function {
                name: Cow::Borrowed("function3"),
                disabled: false,
                bindings: Cow::Borrowed(&[]),
                invoker_name: None,
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
            invoker_name: None,
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
            invoker_name: None,
            invoker: None,
            manifest_dir: None,
            file: None,
        }]);
        assert_eq!(registry.iter().count(), 1);

        assert_eq!(registry.register("id", "not_present"), false);
    }

    #[test]
    fn it_iterates_binding_extensions() {
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
            invoker_name: None,
            invoker: None,
            manifest_dir: None,
            file: None,
        }]);
        assert_eq!(registry.iter_binding_extensions().count(), 1);

        let extensions = registry.iter_binding_extensions().nth(0).unwrap();

        assert_eq!(extensions.0, STORAGE_PACKAGE_NAME);
        assert_eq!(extensions.1, STORAGE_PACKAGE_VERSION);
    }

    #[test]
    fn it_iterates_empty_binding_extensions() {
        let registry = Registry::new(&[&Function {
            name: Cow::Borrowed("function1"),
            disabled: false,
            bindings: Cow::Borrowed(&[Binding::Http(bindings::Http {
                name: Cow::Borrowed("binding1"),
            })]),
            invoker_name: None,
            invoker: None,
            manifest_dir: None,
            file: None,
        }]);
        assert_eq!(registry.iter_binding_extensions().count(), 0);
    }
}
