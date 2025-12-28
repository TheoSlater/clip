use axum::response::sse::{Event, Sse};
use std::{convert::Infallible, time::Duration};
use tokio_stream::{StreamExt, wrappers::IntervalStream};

pub async fn get_connection() -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let connected = tokio_stream::iter(vec![Ok(Event::default().event("connected"))]);
    let keepalive = IntervalStream::new(tokio::time::interval(Duration::from_secs(5)))
        .map(|_| Ok(Event::default().event("ping").data("ping")));

    Sse::new(connected.chain(keepalive))
}
