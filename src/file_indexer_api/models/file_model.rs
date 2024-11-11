use std::collections::HashMap;

use qdrant_client::qdrant::Value;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct FileModel{
    pub name:String
}

impl FileModel{
    pub fn as_map(&self)->HashMap<String,  Value>{
        let mut map: HashMap<String, Value> = HashMap::new();
        map.insert("name".to_string(),self.name.clone().into());
        map
    }

    pub fn from_qdrant_payload(payload: &std::collections::HashMap<String, qdrant_client::qdrant::Value>) -> Result<FileModel, String> {
        payload.get("name")
            .and_then(|name_key| name_key.as_str().map(|name| name.to_string()))
            .map(|name| FileModel { name })
            .ok_or_else(|| "Name field doesn't exist".to_string())
    }
}
