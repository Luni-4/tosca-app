use std::convert::Infallible;
use std::time::Duration;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
};

use futures::stream::Stream;

use serde::Serialize;

use tokio_stream::StreamExt as _;
use tokio_stream::wrappers::BroadcastStream;

use crate::AppState;

const THROTTLE: Duration = Duration::from_secs(1);
const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Serialize, Clone, Copy, PartialEq)]
struct EventData {
    is_light_on: bool,
    temperature: f32,
}

impl EventData {
    const fn new(is_light_on: bool, temperature: f32) -> Self {
        Self {
            is_light_on,
            temperature,
        }
    }
}

pub(crate) async fn event_stream(
    Path(device_id): Path<usize>,
    State(state): State<AppState>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, StatusCode> {
    let devices_receivers = state.devices_receivers.lock().await;

    let receiver = if let Some(receiver) = devices_receivers.get(&device_id) {
        receiver.resubscribe()
    } else {
        #[cfg(feature = "logging")]
        tracing::error!(
            "{}",
            t!("logging.events_nonexistent_device", device_id = device_id)
        );
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
                    tracing::error!("{}", t!("logging.events_failed_reception", e = e));
                    return None;
                }
            };

            // FIXME: The events logic should be provided by the card layer and
            // not hard-coded here.
            let is_light_on = events.bool_events_as_slice().iter().any(|e| e.value);
            // If no temperature event is found, use an unreachable Celsius
            // value to indicate an error.
            let temperature = events
                .f32_events_as_slice()
                .iter()
                .find(|e| e.name == "temperature")
                .map_or(-274.0, |e| e.value);

            let event_data = EventData::new(is_light_on, temperature);

            #[cfg(feature = "logging")]
            tracing::info!("{events}");

            // Skip sending SSE event if the events data has not changed.
            let new_state = Some(event_data);
            if previous_state == new_state {
                return None;
            }
            previous_state = new_state;

            let string_data = match serde_json::to_string(&event_data) {
                Ok(string_data) => string_data,
                Err(e) => {
                    #[cfg(feature = "logging")]
                    tracing::error!("{}", t!("logging.events_failed_serialization", e = e));
                    return None;
                }
            };

            Some(Ok(Event::default()
                .id(device_id.to_string())
                .data(string_data)))
        })
        .throttle(THROTTLE);

    Ok(Sse::new(sse_stream).keep_alive(KeepAlive::default().interval(KEEPALIVE_INTERVAL)))
}
