use super::db_manager::FileVectorDbManager;
use crate::file_indexer_api::models::file_model::FileModel;
use std::sync::Arc;
use tokio::sync::mpsc;

pub async fn index_worker(
    db_manager: Arc<FileVectorDbManager>,
    batch_size: usize,
    receiver: &mut mpsc::Receiver<FileModel>,
) {
    let mut queue: Vec<FileModel> = Vec::new();
    for file in receiver.recv().await {
        queue.push(file);
        if queue.len() > batch_size {
            let db_manager_clone = Arc::clone(&db_manager);
            dispatch_queue(db_manager_clone, &mut queue).await;
        }
    }
}

async fn dispatch_queue(db_manager: Arc<FileVectorDbManager>, queue: &mut Vec<FileModel>) {
    let db_manager = Arc::clone(&db_manager);
    let mut dispatch: Vec<FileModel> = Vec::new();
    dispatch.append(queue);
    tokio::spawn(async move {
        db_manager.insert_many(dispatch);
    });
}
