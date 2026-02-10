use std::collections::HashMap;

use axum::extract::State;
use axum::response::Redirect;

use crate::AppState;
use crate::devices::{DemoLight, Devices, LocalizedHazard, Route, RouteData, RouteMetadata};
use crate::error::{Error, error_with_info};

// Find `tosca` devices in the network.
pub(crate) async fn run_discovery(State(state): State<AppState>) -> Result<Redirect, Error> {
    let mut controller = state.controller.lock().await;

    // Discover devices
    error_with_info(
        &state.env,
        controller.discover().await,
        &t!("discovery_errors.discovery_failed"),
    )?;

    if !controller.devices().is_empty() {
        let len = controller.devices().len();
        let mut devices = Devices::with_capacity(len);
        let mut devices_receivers = HashMap::with_capacity(len);

        // TODO&FIXME: This is a temporary workaround, implemented because we are
        // currently testing only a single light. The proper solution would involve
        // assigning a name to each device and validating its parameters against the
        // provided card to ensure consistency and correctness.
        //
        //
        // A device id is given by the order.
        for (id, device) in controller.devices_mut().iter_mut().enumerate() {
            // Retrieve the hazards associated with this device for each request
            let mut routes = HashMap::new();
            for (id, ri) in device.requests_info().iter().enumerate() {
                // Do not consider state routes.
                if DemoLight::is_state_route(ri.route) {
                    continue;
                }

                let hazards = if ri.hazards.is_empty() {
                    Vec::new()
                } else {
                    ri.hazards
                        .into_iter()
                        .map(|h| LocalizedHazard::new(h.id(), h.category().name()))
                        .collect()
                };

                let metadata = RouteMetadata::new(
                    t!(format!("{}.name", ri.route)),
                    t!(format!("{}.description", ri.route)),
                );
                let data = RouteData::new(id, hazards);

                routes.insert(ri.route.to_string(), Route::new(metadata, data));
            }

            let demo_light = if device.has_events() {
                let receiver = error_with_info(
                    &state.env,
                    device.start_event_receiver(id, 100).await,
                    &t!("discovery_errors.start_device_events_failed"),
                )?;
                devices_receivers.insert(id, receiver);
                DemoLight::with_events(id, routes)
            } else {
                DemoLight::new(id, routes)
            };

            devices.add_device(demo_light);
        }

        *state.devices.lock().await = devices;
        *state.devices_receivers.lock().await = devices_receivers;
    }

    // Redirect to index
    Ok(Redirect::to("/"))
}
