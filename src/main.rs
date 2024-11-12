use file_indexer_api::{models::file_model::FileModel, service::FileVectorDbService};

mod file_indexer_api;
mod vevtor;

#[tokio::main]
async fn main() {
    let url = "http://localhost:6334";
    let service = FileVectorDbService::new(url, 64);

    service.delete_all_collections().await;

    let files: Vec<FileModel> = vec![FileModel {
        name: "joe".to_string(),
        collection: "files".to_string(),
    }];

    if let Err(err) = service.insert_many(files).await {
        println!("Insert error: {}", err);
    }


}
