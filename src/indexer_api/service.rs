use std::sync::Arc;

use tokio::sync::mpsc::{Receiver, Sender};

use super::{
    infrastructure::{db_manager::FileVectorDbManager, index_worker},
    models::search_query_models::VectorQueryModel,
    traits::indexable::{Indexable, IntoPayload},
    util::hashing::string_to_u64,
};

type Collection = String;
type ID = u64;
pub struct VevtorService<T>
where
    T: Indexable + IntoPayload,
{
    db_manager: Arc<FileVectorDbManager>,
    sender: Sender<T>,
}

impl<T> VevtorService<T>
where
    T: Indexable + IntoPayload,
{
    pub fn new(qdrant_url: &str, batch_size: usize) -> Self {
        let db_manager = Arc::new(FileVectorDbManager::new(qdrant_url));

        let db_manager_clone = Arc::clone(&db_manager);
        let (sender, receiver) = tokio::sync::mpsc::channel::<T>(30);
        Self::spawn_index_worker(db_manager_clone, receiver, batch_size);

        Self { db_manager, sender }
    }

    pub async fn add_files(&self, files: Vec<T>) {
        for file in files.into_iter() {
            if let Err(err) = self.sender.send(file).await {
                println!("error sending file: {}", err);
            }
        }
    }

    pub async fn search(
        &self,
        params: &VectorQueryModel,
        top_k: u64,
    ) -> Result<Vec<(T, f32)>, String> {
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

    fn spawn_index_worker(
        db_manager: Arc<FileVectorDbManager>,
        receiver: Receiver<T>,
        batch_size: usize,
    ) {
        tokio::spawn(async move {
            index_worker::index_worker(db_manager, batch_size, receiver).await;
        });
    }
}
