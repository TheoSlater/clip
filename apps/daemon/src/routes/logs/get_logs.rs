use axum::response::sse::{Event, Sse};
use std::convert::Infallible;
use tokio_stream::{StreamExt, wrappers::BroadcastStream};

use crate::logger::subscribe;

pub async fn get_logs() -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let receiver = subscribe();
    let stream = BroadcastStream::new(receiver).filter_map(|result| match result {
        Ok(event) => {
            let data = serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string());
            Some(Ok(Event::default().event("log").data(data)))
        }
        Err(_) => None,
    });

    Sse::new(stream)
}
