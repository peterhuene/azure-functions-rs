use codegen::Function;
use std::collections::hash_map::Iter;
use std::collections::HashMap;

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
                        panic!("Azure Function '{}' has already been registered; make sure all functions have unique names.", func.name);
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
        self.registered.get(id).map(|x| *x)
    }

    pub fn iter(&self) -> Iter<String, &'a Function> {
        self.functions.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
            },
            &Function {
                name: Cow::Borrowed("function2"),
                disabled: false,
                bindings: Cow::Borrowed(&[]),
                invoker_name: None,
                invoker: None,
            },
            &Function {
                name: Cow::Borrowed("function3"),
                disabled: false,
                bindings: Cow::Borrowed(&[]),
                invoker_name: None,
                invoker: None,
            },
        ]);
        assert_eq!(registry.iter().count(), 3);
        assert!(
            registry
                .iter()
                .all(|(k, _)| *k == "function1" || *k == "function2" || *k == "function3")
        );
    }

    #[test]
    fn it_registers_a_function() {
        let mut registry = Registry::new(&[&Function {
            name: Cow::Borrowed("function1"),
            disabled: false,
            bindings: Cow::Borrowed(&[]),
            invoker_name: None,
            invoker: None,
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
        }]);
        assert_eq!(registry.iter().count(), 1);

        assert_eq!(registry.register("id", "not_present"), false);
    }
}
