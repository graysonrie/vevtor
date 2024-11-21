use std::{collections::HashMap, iter::zip};

use qdrant_client::{qdrant::HealthCheckReply, Payload, QdrantError};
use tokio::sync::RwLock;

use crate::{
    indexer_api::traits::indexable::{Indexable, IntoPayload},
    vector_db::{db::api::QdrantApi, embeddings::generator::EmbeddingsGenerator},
};

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

    pub async fn insert_many<T>(&self, entries: Vec<T>) -> Result<(), String>
    where
        T: Indexable + IntoPayload,
    {
        let embeddings = self.generate_embeddings(&entries)?;

        let batches: HashMap<String, Vec<(T, Vec<f32>)>> =
            self.group_entries(zip(entries, embeddings).collect());

        // Optional check?:
        for (collection_name, _) in batches.iter() {
            self.ensure_collection_exists(collection_name)
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
                            let id = file.get_id();
                            let payload: Payload = file.into();

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

    pub async fn search<T>(
        &self,
        query: &str,
        collection: &str,
        top_k: u64,
    ) -> Result<Vec<(T, f32)>, String>
    where
        T: Indexable + IntoPayload,
    {
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
                // Ignore entries that couldn't be parsed from the payload
                if let Ok(model) = T::from_qdrant_payload(&payload) {
                    return Some((model, score));
                }
                None
            })
            .collect())
    }

    fn generate_embeddings<T>(&self, entries: &[T]) -> Result<Vec<Vec<f32>>, String>
    where
        T: Indexable,
    {
        self.generator
            .embed_many(entries.iter().map(|x| x.embed_label()).collect())
            .map_err(|err| format!("Error generating embeddings: {}", err))
    }

    fn group_entries<T>(
        &self,
        zip: Vec<(T, Vec<f32>)>,
    ) -> HashMap<CollectionName, Vec<(T, Vec<f32>)>>
    where
        T: Indexable,
    {
        let mut grouped_entries: HashMap<CollectionName, Vec<(T, Vec<f32>)>> = HashMap::new();

        for (file, embedding) in zip {
            grouped_entries
                .entry(file.collection()) // Use the collection field as the key
                .or_default()
                .push((file, embedding));
        }
        grouped_entries
    }

    fn group_ids(&self, zip: Vec<(CollectionName, ID)>) -> HashMap<CollectionName, Vec<ID>> {
        let mut grouped_ids: HashMap<CollectionName, Vec<ID>> = HashMap::new();

        for (collection, id) in zip {
            grouped_ids
                .entry(collection.clone()) // Use the collection field as the key
                .or_default()
                .push(id);
        }
        grouped_ids
    }

    pub async fn ensure_collection_exists(&self, name: &str) -> Result<(), QdrantError> {
        let name_str = name.to_string();

        if !self.known_collections.read().await.contains(&name_str) {
            // Only refresh and create if the collection is not known
            self.refresh_known_collections().await;
            self.qdrant
                .create_collection(name, self.generator.embedding_dim_len)
                .await?;
            println!("Created collection: {}", name);
        }

        Ok(())
    }

    pub async fn list_collections(&self) -> Vec<String> {
        self.qdrant.list_collections().await
    }

    pub async fn health_check(&self)->Result<HealthCheckReply,QdrantError>{
        self.qdrant.health_check().await
    }

    async fn refresh_known_collections(&self) {
        let mut known_collections = self.known_collections.write().await;
        *known_collections = self.qdrant.list_collections().await;
    }
}
