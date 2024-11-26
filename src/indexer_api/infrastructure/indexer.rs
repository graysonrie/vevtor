use tokio::sync::mpsc::Sender;

use crate::{indexer_api::traits::indexable::IntoPayload, Indexable};

#[derive(Clone)]
pub struct Indexer<T>
where
    T: Indexable + IntoPayload,
{
    sender: Sender<T>,
}

impl<T> Indexer<T>
where
    T: Indexable + IntoPayload,
{
    pub fn new(sender: Sender<T>) -> Self {
        Self { sender }
    }

    pub async fn index(&self, items: Vec<T>) {
        for item in items.into_iter() {
            if let Err(err) = self.sender.send(item).await {
                println!("Indexer: Error sending item: {}", err);
            }
        }
    }
}
