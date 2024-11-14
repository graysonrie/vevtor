mod indexer_api;
mod tests;
mod vector_db;

pub use indexer_api::models::search_query_models::VectorQueryModel;
pub use indexer_api::service::VevtorService;
pub use indexer_api::traits::indexable::Indexable;
pub use indexable_macro::Indexable;