use std::collections::HashMap;

use axum::extract::State;
use axum::response::Redirect;

use crate::devices::{DemoLight, Devices, LocalizedHazard};
use crate::error::{error_with_info, Error};
use crate::AppState;

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
            let mut hazards = HashMap::new();
            for ri in device.requests_info() {
                if ri.hazards.is_empty() {
                    continue;
                }

                hazards.insert(
                    ri.route.to_string(),
                    ri.hazards
                        .into_iter()
                        .map(|h| LocalizedHazard::new(h.id(), h.category().name()))
                        .collect(),
                );
            }

            let demo_light = if device.has_events() {
                let receiver = error_with_info(
                    &state.env,
                    device.start_event_receiver(id, 100).await,
                    &t!("discovery_errors.start_device_events_failed"),
                )?;
                devices_receivers.insert(id, receiver);
                DemoLight::with_events(id, hazards)
            } else {
                DemoLight::new(id, hazards)
            };

            devices.add_device(demo_light);
        }

        *state.devices.lock().await = devices;
        *state.devices_receivers.lock().await = devices_receivers;
    }

    // Redirect to index
    Ok(Redirect::to("/"))
}
