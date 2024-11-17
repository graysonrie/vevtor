#[cfg(test)]
mod test {
    use std::time::Duration;

    use vevtor::{VectorQueryModel, VevtorService};

    use crate::file_model::FileModel;

    #[tokio::test]
    async fn test() {
        let service = VevtorService::new("http://127.0.0.1:6334");

        service.delete_all_collections().await;

        let mut files: Vec<FileModel> = Vec::new();

        for _ in 0..10 {
            let model = FileModel {
                name: "test".to_string(),
                parent_dir: "parent".to_string(),
                collection: "files".to_string(),
            };
            files.push(model);
        }

        let indexer = service.spawn_index_worker::<FileModel>(32, 32);

        indexer.index(files).await;

        tokio::time::sleep(Duration::from_secs(5)).await;

        match service
            .search::<FileModel>(
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
}
