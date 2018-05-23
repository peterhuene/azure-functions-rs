use codegen::Function;
use std::collections::hash_map::Iter;
use std::collections::HashMap;

pub struct Registry {
    functions: HashMap<&'static str, &'static Function>,
    registered: HashMap<String, &'static Function>,
}

impl Registry {
    pub fn new(functions: &[&'static Function]) -> Registry {
        Registry {
            functions: functions
                .iter()
                .by_ref()
                .fold(HashMap::new(), |mut map, func| {
                    if map.insert(func.name, func).is_some() {
                        panic!("Duplicate function name present.");
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

    pub fn get(&self, id: &str) -> Option<&'static Function> {
        self.registered.get(id).map(|x| *x)
    }

    pub fn iter(&self) -> Iter<&'static str, &'static Function> {
        self.functions.iter()
    }
}
