

use serde::{Deserialize, Serialize};
use vevtor::Indexable;

#[derive(Serialize, Deserialize, Clone, Debug, Indexable)]
#[serde(rename_all = "PascalCase")]
#[indexable(
    id_field = "name",
    collection_field = "collection",
    embed_field = "name"
)]
pub struct FileModel {
    pub name: String,
    pub parent_dir: String,
    pub collection: String,
}
