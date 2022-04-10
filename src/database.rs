use serde_json::{json, Value};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::sync::{Arc, Mutex};

pub struct Database {
    data: HashMap<String, HashMap<String, Arc<Value>>>,
}

impl Database {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    fn insert(&mut self, path: &str, json: Value) -> Arc<Value> {
        let id = random_uuid();
        let value = add_id(&json, &id);
        let arc = Arc::new(value);

        if let Some(map) = self.data.get_mut(path) {
            map.insert(id, arc.clone());
        } else {
            let mut map = HashMap::new();
            map.insert(id, arc.clone());
            self.data.insert(path.to_string(), map);
        };

        arc
    }

    fn get_all(&self, path: &str) -> Value {
        let values: Vec<Value> = if let Some(map) = self.data.get(path) {
            map.values()
                .map(|v| v.to_owned())
                .map(|x| x.as_ref().to_owned())
                .collect::<Vec<_>>()
        } else {
            vec![]
        };
        json!({ "items": values })
    }

    pub fn get_by_id(&self, path: &str, id: &str) -> Option<Value> {
        let map = self.data.get(path)?;
        map.get(id).map(|a| a.to_owned().as_ref().to_owned())
    }
}

fn random_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn add_id(json: &Value, uuid: &str) -> Value {
    let mut value: Value = json.to_owned();
    if let Value::Object(ref mut map) = value {
        map.insert("id".to_owned(), Value::String(uuid.to_string()));
    }
    value
}

pub type ConcurrentDatabase = Arc<Mutex<Database>>;

pub trait DatabaseAccess {
    fn new() -> Self;
    fn insert(&mut self, path: &str, json: Value) -> Result<Arc<Value>, DatabaseError>;
    /// Tries to get single item with id (in last component), otherwise gets all from path
    fn get(&self, path: &str) -> Result<Value, DatabaseError>;
    fn get_all(&self, path: &str) -> Result<Value, DatabaseError>;
    fn get_by_id(&self, path: &str, id: &str) -> Result<Option<Value>, DatabaseError>;
}

impl DatabaseAccess for ConcurrentDatabase {
    fn new() -> ConcurrentDatabase {
        Arc::new(Mutex::new(Database::new()))
    }

    fn insert(&mut self, path: &str, json: Value) -> Result<Arc<Value>, DatabaseError> {
        let mut database = self
            .lock()
            .map_err(|_| DatabaseError::new("Cannot obtain lock"))?;
        Ok(database.insert(path, json))
    }

    fn get(&self, path: &str) -> Result<Value, DatabaseError> {
        let result = match path.split('/').collect::<Vec<_>>().as_slice() {
            [parent @ .., id] => match self.get_by_id(&parent.join("/"), *id)? {
                Some(value) => value,
                None => self.get_all(path)?,
            },
            _ => self.get_all(path)?,
        };
        Ok(result)
    }

    fn get_all(&self, path: &str) -> Result<Value, DatabaseError> {
        let database = self
            .lock()
            .map_err(|_| DatabaseError::new("Cannot obtain lock"))?;
        Ok(database.get_all(path))
    }

    fn get_by_id(&self, path: &str, id: &str) -> Result<Option<Value>, DatabaseError> {
        let database = self
            .lock()
            .map_err(|_| DatabaseError::new("Cannot obtain lock"))?;
        Ok(database.get_by_id(path, id))
    }
}

#[derive(Debug)]
pub struct DatabaseError {
    pub message: String,
}

impl DatabaseError {
    fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl Display for DatabaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::Database;
    use serde_json::{json, Value};

    #[test]
    fn get_all_should_return_empty_json_if_no_data_inserted() {
        let database = Database::new();
        let value = database.get_all("/api/v1/persons");
        assert_eq!(value, json!({"items": []}))
    }

    #[test]
    fn should_insert_data_and_return_inserted() {
        let mut database = Database::new();
        let result = database.insert(
            "/api/v1/persons",
            json!({"firstName": "John", "lastName": "Doe"}),
        );

        assert!(result.get("id").is_some());
        assert_eq!(
            result.get("firstName"),
            Some(&Value::String("John".to_string()))
        );
        assert_eq!(
            result.get("lastName"),
            Some(&Value::String("Doe".to_string()))
        );
    }

    #[test]
    fn get_all_should_return_inserted() {
        let mut database = Database::new();
        database.insert(
            "/api/v1/persons",
            json!({"firstName": "John", "lastName": "Doe"}),
        );

        let value = database.get_all("/api/v1/persons");
        let first = value.get("items").and_then(|v| v.get(0)).unwrap();
        assert_eq!(
            first.get("firstName"),
            Some(&Value::String("John".to_string()))
        );
        assert_eq!(
            first.get("lastName"),
            Some(&Value::String("Doe".to_string()))
        );
    }

    #[test]
    fn get_by_id_should_return_inserted() {
        let mut database = Database::new();
        let inserted = database.insert(
            "/api/v1/persons",
            json!({"firstName": "John", "lastName": "Doe"}),
        );
        let id = inserted.get("id").unwrap().as_str().unwrap();
        let value = database.get_by_id("/api/v1/persons", id);
        assert!(value.is_some());
        assert_eq!(inserted.as_ref(), &value.unwrap());
    }
}
