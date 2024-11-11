mod file_indexer_api;
mod vevtor;

#[tokio::main]
async fn main() {
    let vector_file_service = file_indexer_api::service::VectorFileIndexService::new();

    vector_file_service.reset_all().await;
}
