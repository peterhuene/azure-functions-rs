use codegen::Function;
use std::collections::hash_map::Iter;
use std::collections::HashMap;

pub struct Registry<'a> {
    functions: HashMap<String, &'a Function>,
    registered: HashMap<String, &'a Function>,
}

impl Registry<'a> {
    pub fn new(functions: &[&'a Function]) -> Registry<'a> {
        Registry {
            functions: functions
                .iter()
                .by_ref()
                .fold(HashMap::new(), |mut map, func| {
                    if map.insert(func.name.clone().into_owned(), func).is_some() {
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

    pub fn get(&self, id: &str) -> Option<&'a Function> {
        self.registered.get(id).map(|x| *x)
    }

    pub fn iter(&self) -> Iter<String, &'a Function> {
        self.functions.iter()
    }
}
