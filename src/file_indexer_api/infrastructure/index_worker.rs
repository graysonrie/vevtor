use super::db_manager::FileVectorDbManager;
use crate::file_indexer_api::models::file_model::FileModel;
use std::sync::Arc;
use tokio::sync::mpsc;

pub async fn index_worker(
    db_manager: Arc<FileVectorDbManager>,
    batch_size: usize,
    mut receiver: mpsc::Receiver<FileModel>,
) {
    let mut queue: Vec<FileModel> = Vec::new();
    println!("open");
    while let Some(file) = receiver.recv().await {
        queue.push(file);
        if queue.len() >= batch_size {
            let db_manager_clone = Arc::clone(&db_manager);
            dispatch_queue(db_manager_clone, &mut queue).await;
        }
    }
    println!("closed");
    if !queue.is_empty() {
        dispatch_queue(db_manager, &mut queue).await;
    }
}

async fn dispatch_queue(db_manager: Arc<FileVectorDbManager>, queue: &mut Vec<FileModel>) {
    println!("dispatching queue");
    let mut dispatch: Vec<FileModel> = Vec::new();
    dispatch.append(queue);
    if let Err(err) = db_manager.insert_many(dispatch).await {
        println!("Error inserting files: {}", err);
    }
}
