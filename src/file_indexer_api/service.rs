use std::sync::Arc;

use tokio::sync::RwLock;

use super::{
    infrastructure::db_manager::{self, FileVectorDbManager},
    models::{file_model::FileModel, search_query_models::VectorQueryModel},
    util::vec::extract_vec_from_lock,
};

pub struct FileVectorDbService {
    db_manager: Arc<FileVectorDbManager>,
    queue: Arc<RwLock<Vec<FileModel>>>,
    batch_size: usize,
}

/// Manages the worker instance
impl FileVectorDbService {
    pub fn new(qdrant_url: &str, batch_size: usize) -> Self {
        let db_manager = Arc::new(FileVectorDbManager::new(qdrant_url));
        Self {
            db_manager,
            queue: Arc::new(RwLock::new(Vec::new())),
            batch_size,
        }
    }

    pub async fn add_files(&self, files: &mut Vec<FileModel>) {
        let mut queue = self.queue.write().await;
        queue.append(files);
        self.check_to_dispatch_queue().await;
    }

    pub async fn search(
        &self,
        params: &VectorQueryModel,
        top_k: u64,
    ) -> Result<Vec<(FileModel, f32)>, String> {
        self.db_manager
            .search(&params.query, &params.collection, top_k)
            .await
    }

    pub async fn delete_all_collections(&self) {
        self.db_manager.reset_all().await;
    }

    async fn check_to_dispatch_queue(&self) {
        let len = self.queue.read().await.len();
        if len > self.batch_size {
            self.dispatch_queue().await;
        }
    }

    async fn dispatch_queue(&self) {
        let files = extract_vec_from_lock(Arc::clone(&self.queue)).await;
        let db_manager = Arc::clone(&self.db_manager);
        tokio::spawn(async move {
            db_manager.insert_many(files);
        });
    }
}
