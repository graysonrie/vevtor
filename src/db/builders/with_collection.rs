use std::collections::HashMap;

use crate::db::api::Embeddings;
use qdrant_client::qdrant::vectors::VectorsOptions;
use qdrant_client::qdrant::{
    CollectionOperationResponse, CreateCollectionBuilder, Distance, PointsOperationResponse,
    ScoredPoint, SearchPointsBuilder, Vector, VectorParamsBuilder,
};
use qdrant_client::qdrant::{PointStruct, UpsertPointsBuilder};
use qdrant_client::{Payload, Qdrant, QdrantError};
use uuid::Uuid;

type EmbeddingResult = (HashMap<String, qdrant_client::qdrant::Value>, f32);

pub struct WithCollectionBuilder<'a> {
    client: &'a Qdrant,
    collection: String,
}

impl<'a> WithCollectionBuilder<'a> {
    pub fn new(client: &'a Qdrant, collection: &str) -> Self {
        Self {
            client,
            collection: collection.to_string(),
        }
    }

    pub async fn insert_many(
        &self,
        data: Vec<(Embeddings, HashMap<&str, qdrant_client::qdrant::Value>, u64)>,
    ) {
        for (embedding, payload, id) in data.into_iter() {
            _ = self.insert(embedding, payload, id).await;
        }
    }

    pub async fn insert(
        &self,
        embeddings: Embeddings,
        payload: HashMap<&str, qdrant_client::qdrant::Value>,
        id: u64,
    ) -> Result<PointsOperationResponse, QdrantError> {
        let points = vec![PointStruct::new(
            id,         // Uniqe point ID
            embeddings, // Vector to upsert
            // Attached payload
            payload,
        )];
        self.client
            .upsert_points(UpsertPointsBuilder::new(&self.collection, points))
            .await
    }

    pub async fn search(
        &self,
        embedding: Embeddings,
        top_k: u64,
    ) -> Result<Vec<EmbeddingResult>, QdrantError> {
        let search_request =
            SearchPointsBuilder::new(&self.collection, embedding, top_k).with_payload(true);

        self.client
            .search_points(search_request)
            .await
            .map(|response| {
                response
                    .result
                    .into_iter()
                    .map(|result| {
                        let score = result.score;
                        (result.payload, score)
                    })
                    .collect()
            })
    }
}
