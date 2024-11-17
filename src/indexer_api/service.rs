pub use super::infrastructure::indexer::Indexer;
use super::{
    infrastructure::{db_manager::FileVectorDbManager, index_worker},
    models::search_query_models::VectorQueryModel,
    traits::indexable::{Indexable, IntoPayload},
    util::hashing::string_to_u64,
};
use std::sync::Arc;

type Collection = String;
type ID = u64;
pub struct VevtorService {
    db_manager: Arc<FileVectorDbManager>,
}

impl VevtorService {
    pub fn new(qdrant_url: &str) -> Self {
        let db_manager = Arc::new(FileVectorDbManager::new(qdrant_url));
        Self { db_manager }
    }

    pub async fn search<T>(
        &self,
        params: &VectorQueryModel,
        top_k: u64,
    ) -> Result<Vec<(T, f32)>, String>
    where
        T: Indexable + IntoPayload,
    {
        self.db_manager
            .search::<T>(&params.query, &params.collection, top_k)
            .await
    }

    pub async fn list_collections(&self) -> Vec<String> {
        self.db_manager.list_collections().await
    }

    pub async fn delete_all_collections(&self) {
        self.db_manager.reset_all().await;
    }

    pub async fn ensure_collection_exists(&self, name: &str) -> Result<(), String> {
        self.db_manager
            .ensure_collection_exists(name)
            .await
            .map_err(|err| format!("Error when ensuring that collection exists: {}", err))
    }

    pub async fn delete_by_str_id(&self, ids: Vec<(Collection, String)>) {
        // uses the same hash function that the macro uses
        self.db_manager
            .delete_many(
                ids.into_iter()
                    .map(|(x, y)| (x, string_to_u64(&y)))
                    .collect(),
            )
            .await
    }

    pub async fn delete_by_id(&self, ids: Vec<(Collection, ID)>) {
        self.db_manager.delete_many(ids).await
    }

    pub fn spawn_index_worker<T>(&self, batch_size: usize, buffer_size: usize) -> Indexer<T>
    where
        T: Indexable + IntoPayload,
    {
        let db_manager_clone = Arc::clone(&self.db_manager);
        let (sender, receiver) = tokio::sync::mpsc::channel::<T>(buffer_size);
        tokio::spawn(async move {
            index_worker::index_worker(db_manager_clone, batch_size, receiver).await;
        });
        Indexer::new(sender)
    }
}
