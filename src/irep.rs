use json::object;
use json::JsonValue;
use std::collections::HashMap;
#[derive(Clone, Debug, Default)]
pub struct Irept {
    // TODO: This should be references and not copies!
    pub id: String,
    pub subt: Vec<Irept>,
    pub named_subt: HashMap<String, Irept>,
    pub comments: HashMap<String, Irept>,
}

impl Irept {
    pub fn get_nil() -> Self {
        Irept::from("nil")
    }
}

impl From<&Irept> for JsonValue {
    fn from(data: &Irept) -> Self {
        let mut obj = object! {id: data.id.clone()};

        let mut sub_vec: Vec<JsonValue> = Vec::new();
        for sub in &data.subt {
            sub_vec.push(JsonValue::from(sub));
        }
        if !sub_vec.is_empty() {
            obj["subt"] = JsonValue::from(sub_vec);
        }

        for (k, v) in &data.named_subt {
            obj[k] = JsonValue::from(v);
        }

        for (k, v) in &data.comments {
            obj[k] = JsonValue::from(v);
        }
        obj
    }
}

impl std::hash::Hash for Irept {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        for irep in &self.subt {
            irep.hash(state);
        }
        for (name, irep) in &self.named_subt {
            name.hash(state);
            irep.hash(state);
        }
        for (name, irep) in &self.comments {
            name.hash(state);
            irep.hash(state);
        }
    }
}

impl PartialEq for Irept {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.subt == other.subt
            && self.named_subt == other.named_subt
            && self.comments == other.comments
    }
}
impl Eq for Irept {}

impl std::fmt::Display for Irept {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let as_json = JsonValue::from(self);
        write!(f, "{}", json::stringify_pretty(as_json, 4))
    }
}

impl From<&String> for Irept {
    fn from(data: &String) -> Self {
        Irept {
            id: data.clone(),
            ..Default::default()
        }
    }
}

impl From<String> for Irept {
    fn from(data: String) -> Self {
        Irept {
            id: data,
            ..Default::default()
        }
    }
}

impl From<&str> for Irept {
    fn from(data: &str) -> Self {
        Irept {
            id: data.to_string(),
            ..Default::default()
        }
    }
}

impl Irept {}
