use std::collections::HashMap;

use qdrant_client::qdrant::Value;

pub trait FromQdrantPayload: Sized {
    fn from_qdrant_payload(
        payload: &HashMap<String, Value>,
    ) -> Result<Self, String>;
}
