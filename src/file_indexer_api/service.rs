use std::iter::zip;

use crate::vevtor::{
    db::api::QdrantApi,
    embeddings::generator::{self, EmbeddingsGenerator},
};

use super::{models::file_model::FileModel, util::hashing};

const FILES_COLLECTION: &str = "files";
pub struct VectorFileIndexService {
    qdrant: QdrantApi,
    generator: EmbeddingsGenerator,
}

impl VectorFileIndexService {
    pub fn new() -> Self {
        let qdrant = QdrantApi::new("http://localhost:6334");
        let generator = EmbeddingsGenerator::new();
        Self { qdrant, generator }
    }

    pub async fn reset_all(&self) {
        self.qdrant
            .delete_collection(FILES_COLLECTION)
            .await
            .unwrap();
    }

    pub async fn insert_many(&self, files: &[FileModel]) -> Result<(), String> {
        let embeddings = self
            .generator
            .embed_many(files.iter().map(|x| x.name.as_str()).collect())
            .map_err(|err| format!("Error generating embeeddings: {}", err))?;

        let zip: Vec<(&FileModel, Vec<f32>)> = zip(files, embeddings).collect();

        self.qdrant
            .with_collection(FILES_COLLECTION)
            .insert_many(
                zip.into_iter()
                    .map(|(file, embeddings)| {
                        let payload = file.as_map();
                        let id = hashing::string_to_u64(&file.name); // name of the document is the ID

                        (embeddings, payload, id)
                    })
                    .collect(),
            )
            .await;

        Ok(())
    }

    pub async fn search(&self, query: &str, top_k: u64) -> Result<Vec<(FileModel, f32)>, String> {
        let test = self.generator.embed(query).unwrap();

        let search: Vec<(
            std::collections::HashMap<String, qdrant_client::qdrant::Value>,
            f32,
        )> = self
            .qdrant
            .with_collection("test")
            .search(test, top_k)
            .await
            .map_err(|err| format!("Search error: {}", err))?;

        Ok(search
            .into_iter()
            .map(|(payload, score)| {
                if let Ok(model) = FileModel::from_qdrant_payload(&payload){
                    return Some((model, score));
                }
                None
            }).filter_map(|x| x)
            .collect())
    }
}
