use std::collections::HashMap;

use qdrant_client::qdrant::Value;

pub trait IntoPayload: Into<qdrant_client::Payload> {}

// Implement `IntoPayload` for any type `T` that implements both `Indexable` and `Into<Payload>`
impl<T> IntoPayload for T where T: Indexable + Into<qdrant_client::Payload> {}

pub trait Indexable: Send + Sync + Sized + 'static {
    fn as_map(&self) -> HashMap<String, Value>;

    fn get_id(&self) -> u64;

    fn collection(&self) -> String;

    fn embed_label(&self) -> &str;

    fn from_qdrant_payload(payload: &HashMap<String, Value>) -> Result<Self, String>;
}
