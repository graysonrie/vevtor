use std::time::Duration;

use file_indexer_api::{
    models::{file_model::FileModel, search_query_models::VectorQueryModel},
    service::FileVectorDbService,
};

mod file_indexer_api;
mod vevtor;

#[tokio::main]
async fn main() {
    let url = "http://127.0.0.1:6334";
    let service = FileVectorDbService::new(url, 8);

    service.delete_all_collections().await;

    let mut files: Vec<FileModel> = Vec::new();

    for _ in 0..10 {
        let model = FileModel {
            name: "test".to_string(),
            collection: "files".to_string(),
        };
        files.push(model);
    }

    service.add_files(files).await;

    tokio::time::sleep(Duration::from_secs(5)).await;

    match service
        .search(
            &VectorQueryModel {
                collection: "files".to_string(),
                query: "test".to_string(),
            },
            10,
        )
        .await
    {
        Ok(val) => println!("Search results: {:?}", val),
        Err(err) => println!("Search error: {}", err),
    }
}
