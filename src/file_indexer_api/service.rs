use std::sync::Arc;

use tokio::sync::mpsc::{Receiver, Sender};

use super::{
    infrastructure::{db_manager::FileVectorDbManager, index_worker},
    models::{file_model::FileModel, search_query_models::VectorQueryModel},
};

pub struct FileVectorDbService {
    db_manager: Arc<FileVectorDbManager>,
    sender: Sender<FileModel>,
}

impl FileVectorDbService {
    pub fn new(qdrant_url: &str, batch_size: usize) -> Self {
        let db_manager = Arc::new(FileVectorDbManager::new(qdrant_url));

        let db_manager_clone = Arc::clone(&db_manager);
        let (sender, receiver) = tokio::sync::mpsc::channel::<FileModel>(30);
        FileVectorDbService::spawn_index_worker(db_manager_clone, receiver, batch_size);

        Self { db_manager, sender }
    }

    pub async fn add_files(&self, files: Vec<FileModel>) {
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
    ) -> Result<Vec<(FileModel, f32)>, String> {
        self.db_manager
            .search(&params.query, &params.collection, top_k)
            .await
    }

    pub async fn delete_all_collections(&self) {
        self.db_manager.reset_all().await;
    }

    fn spawn_index_worker(
        db_manager: Arc<FileVectorDbManager>,
        receiver: Receiver<FileModel>,
        batch_size: usize,
    ) {
        tokio::spawn(async move {
            index_worker::index_worker(db_manager, batch_size, receiver).await;
        });
    }
}
