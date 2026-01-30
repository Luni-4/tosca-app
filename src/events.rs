use std::convert::Infallible;
use std::time::Duration;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
};

use futures::stream::Stream;

use tokio_stream::StreamExt as _;
use tokio_stream::wrappers::BroadcastStream;

use crate::AppState;

const THROTTLE: Duration = Duration::from_secs(1);
const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(1);

pub(crate) async fn event_stream(
    Path(device_id): Path<usize>,
    State(state): State<AppState>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, StatusCode> {
    let devices_receivers = state.devices_receivers.lock().await;

    let receiver = if let Some(receiver) = devices_receivers.get(&device_id) {
        receiver.resubscribe()
    } else {
        #[cfg(feature = "logging")]
        tracing::error!("Events: {}", format!("Device `{device_id}` does not exist"));
        // The browser receives no output, the server continues running,
        // and the issue is logged.
        //
        // This allows the user to continue using the application even if
        // the device is not found.
        return Err(StatusCode::NO_CONTENT);
    };

    // Release the lock.
    drop(devices_receivers);

    // Track the last sent device state for this SSE connection.
    let mut previous_state = None;

    // Convert the stream into SSE events
    let sse_stream = BroadcastStream::new(receiver)
        .filter_map(move |events| {
            let events = match events {
                Ok(events) => events,
                Err(e) => {
                    #[cfg(feature = "logging")]
                    tracing::error!("Failed to receive events: {e}");
                    return None;
                }
            };

            // FIXME: This logic should be provided by the card layer and
            // not hard-coded here.
            let light_status = events.bool_events_as_slice().iter().any(|e| e.value);

            #[cfg(feature = "logging")]
            tracing::info!("{events}");

            // Skip sending SSE event if the light state hasn’t changed.
            if previous_state == Some(light_status) {
                return None;
            }
            previous_state = Some(light_status);

            Some(Ok(Event::default()
                .id(device_id.to_string())
                .data(light_status.to_string())))
        })
        .throttle(THROTTLE);

    Ok(Sse::new(sse_stream).keep_alive(KeepAlive::default().interval(KEEPALIVE_INTERVAL)))
}
