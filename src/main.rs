use std::collections::HashMap;

use db::api::QdrantApi;
use embeddings::generator::EmbeddingsGenerator;
use util::hashing;

mod db;
mod embeddings;
mod util;

#[tokio::main]
async fn main() {
    let docs: Vec<&str> = vec!["I am doc", "me too"];
    let generator = EmbeddingsGenerator::new();

    let db_client = QdrantApi::new("http://localhost:6334");

    let embeddings = generator.embed_named(docs).unwrap();

    db_client.delete_collection("test").await.unwrap();

    println!("creating collection");
    _ = db_client
        .create_collection("test", generator.embedding_dim_len)
        .await;

    println!("inserting into collection");

    db_client
        .with_collection("test")
        .insert_many(
            embeddings
                .into_iter()
                .map(|(x, y)| {
                    let mut payload: HashMap<&str, qdrant_client::qdrant::Value> = HashMap::new();
                    let id = hashing::string_to_u64(&y); // name of the document is the ID
                    payload.insert("name", y.into());
                    (x, payload, id)
                })
                .collect(),
        )
        .await;

    let test = generator.embed("i am test").unwrap();

    println!("searching");

    let search = db_client
        .with_collection("test")
        .search(test, 5)
        .await
        .unwrap();

    println!("Found {} results", search.len());

    for item in search.iter() {
        println!("{:?}", item);
    }
}
