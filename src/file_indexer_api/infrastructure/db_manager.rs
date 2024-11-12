use std::{collections::HashMap, iter::zip};

use qdrant_client::QdrantError;
use tokio::sync::RwLock;

use crate::vevtor::{db::api::QdrantApi, embeddings::generator::EmbeddingsGenerator};

use super::super::models::file_model::FileModel;
pub struct FileVectorDbManager {
    qdrant: QdrantApi,
    generator: EmbeddingsGenerator,
    known_collections: RwLock<Vec<String>>,
}

type CollectionName = String;
type ID = u64;
impl FileVectorDbManager {
    pub fn new(url: &str) -> Self {
        let qdrant = QdrantApi::new(url);
        let generator = EmbeddingsGenerator::new();
        Self {
            qdrant,
            generator,
            known_collections: RwLock::new(Vec::new()),
        }
    }

    pub async fn reset_all(&self) {
        let collections = self.qdrant.list_collections().await;
        self.qdrant
            .delete_collections(&collections.iter().map(|x| x.as_str()).collect())
            .await;
    }

    pub async fn insert_many(&self, files: Vec<FileModel>) -> Result<(), String> {
        let embeddings = self.generate_embeddings(&files)?;

        let batches = self.group_files(zip(files, embeddings).collect());

        // Optional check?:
        for (collection_name, _) in batches.iter() {
            self.ensure_collection_exists(collection_name, self.generator.embedding_dim_len)
                .await
                .map_err(|err| format!("Error ensuring collection exists: {}", err))?;
        }

        for (collection_name, file_group) in batches {
            self.qdrant
                .with_collection(&collection_name) // Use specific collection
                .insert_many(
                    file_group
                        .into_iter()
                        .map(|(file, embeddings)| {
                            let payload = file.as_map();
                            let id = file.get_id();

                            (embeddings, payload, id)
                        })
                        .collect(),
                )
                .await;
        }

        Ok(())
    }

    pub async fn delete_many(&self, ids: Vec<(CollectionName, ID)>) {
        let groups = self.group_ids(ids);

        for (collection, ids) in groups {
            if let Err(err) = self
                .qdrant
                .with_collection(&collection)
                .remove_many(ids)
                .await
            {
                println!(
                    "Error deleting ids from collection '{}': {}",
                    collection, err
                )
            }
        }
    }

    pub async fn search(
        &self,
        query: &str,
        collection: &str,
        top_k: u64,
    ) -> Result<Vec<(FileModel, f32)>, String> {
        let test = self.generator.embed(query).unwrap();

        let search: Vec<(
            std::collections::HashMap<String, qdrant_client::qdrant::Value>,
            f32,
        )> = self
            .qdrant
            .with_collection(collection)
            .search(test, top_k)
            .await
            .map_err(|err| format!("Search error: {}", err))?;

        Ok(search
            .into_iter()
            .filter_map(|(payload, score)| {
                // Ignore files that couldn't be parsed from the payload
                if let Ok(model) = FileModel::from_qdrant_payload(&payload, collection.to_string())
                {
                    return Some((model, score));
                }
                None
            })
            .collect())
    }

    fn generate_embeddings(&self, files: &[FileModel]) -> Result<Vec<Vec<f32>>, String> {
        self.generator
            .embed_many(files.iter().map(|x| x.name.as_str()).collect())
            .map_err(|err| format!("Error generating embeddings: {}", err))
    }

    fn group_files(
        &self,
        zip: Vec<(FileModel, Vec<f32>)>,
    ) -> HashMap<CollectionName, Vec<(FileModel, Vec<f32>)>> {
        let mut grouped_files: HashMap<CollectionName, Vec<(FileModel, Vec<f32>)>> = HashMap::new();

        for (file, embedding) in zip {
            grouped_files
                .entry(file.collection.clone()) // Use the collection field as the key
                .or_insert_with(Vec::new)
                .push((file, embedding));
        }
        grouped_files
    }

    fn group_ids(&self, zip: Vec<(CollectionName, ID)>) -> HashMap<CollectionName, Vec<ID>> {
        let mut grouped_ids: HashMap<CollectionName, Vec<ID>> = HashMap::new();

        for (collection, id) in zip {
            grouped_ids
                .entry(collection.clone()) // Use the collection field as the key
                .or_insert_with(Vec::new)
                .push(id);
        }
        grouped_ids
    }

    async fn ensure_collection_exists(
        &self,
        name: &str,
        num_features: u64,
    ) -> Result<(), QdrantError> {
        let name_str = name.to_string();
        let mut contains = false;
        {
            contains = self.known_collections.read().await.contains(&name_str);
        }
        if !contains {
            self.refresh_known_collections().await;
            self.qdrant.create_collection(name, num_features).await?;
            println!("Created collection: {}", name);
        }
        Ok(())
    }

    async fn refresh_known_collections(&self) {
        let mut known_collections = self.known_collections.write().await;
        *known_collections = self.qdrant.list_collections().await;
    }
}
