use crate::indexer_api::traits::indexable::Indexable;
use qdrant_client::qdrant::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use twox_hash::XxHash64;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct FileModel {
    pub name: String,
    pub parent_dir: String,
    pub collection: String,
}

impl Indexable for FileModel {

    fn as_map(&self) -> HashMap<String, qdrant_client::qdrant::Value> {
        let value: serde_json::Value = serde_json::to_value(self).expect("Serialization failed");

        if let serde_json::Value::Object(map) = value {
            map.into_iter().map(|(k, v)| (k, v.into())).collect()
        } else {
            HashMap::new()
        }
    }

    fn get_id(&self) -> u64 {
        string_to_u64(&self.name)
    }

    fn embed_label(&self) -> &str {
        &self.name
    }

    fn collection(&self) -> String {
        self.collection.to_string()
    }
}

impl From<FileModel> for qdrant_client::Payload {
    fn from(val: FileModel) -> Self {
        // Use `as_map` to create a HashMap and convert it to `Payload`
        qdrant_client::Payload::from(val.as_map())
    }
}

impl From<qdrant_client::Payload> for FileModel{
    fn from(payload:qdrant_client::Payload) -> Self{
        let json_value: serde_json::Value = serde_json::to_value(payload)
            .expect("Failed to convert payload to JSON");
    
        let file_model: FileModel = serde_json::from_value(json_value)
        .expect("Failed to convert JSON to Indexable");
    
        file_model
    }
}

impl From<std::collections::HashMap<String, qdrant_client::qdrant::Value>> for FileModel{
    fn from(payload:std::collections::HashMap<String, qdrant_client::qdrant::Value>) -> Self{
        let json_value: serde_json::Value = serde_json::to_value(payload)
            .expect("Failed to convert payload to JSON");
    
        let file_model: FileModel = serde_json::from_value(json_value)
        .expect("Failed to convert JSON to Indexable");
    
        file_model
    }
}

pub fn string_to_u64(s: &str) -> u64 {
    let mut hasher = XxHash64::default();
    s.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod test {
    use crate::tests::file_model::FileModel;

    #[tokio::test]
    async fn test() {
        let model = FileModel {
            name: "name".to_string(),
            parent_dir: "dir".to_string(),
            collection: "test".to_string(),
        };
        let payload:qdrant_client::Payload = model.into();
    }
}
