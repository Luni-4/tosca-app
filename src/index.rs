use std::borrow::Cow;

use axum::extract::State;
use axum::response::Html;

use serde::Serialize;

use crate::AppState;
use crate::devices::Devices;
use crate::error::{Error, error_with_info};
use crate::layout::RenderLayout;

#[derive(Serialize)]
struct RenderMessages<'a> {
    no_devices_message: Cow<'a, str>,
}

impl RenderMessages<'_> {
    #[inline]
    fn new() -> Self {
        Self {
            no_devices_message: t!("labels.no_devices"),
        }
    }
}

#[derive(Serialize)]
struct RenderRoutes<'a> {
    discovery_route: Cow<'a, str>,
}

impl RenderRoutes<'_> {
    #[inline]
    fn new() -> Self {
        Self {
            discovery_route: t!("routes.discovery"),
        }
    }
}

#[derive(Serialize)]
struct RenderButtons<'a> {
    discovery_button: Cow<'a, str>,
}

impl RenderButtons<'_> {
    #[inline]
    fn new() -> Self {
        Self {
            discovery_button: t!("buttons.discovery"),
        }
    }
}

#[derive(Serialize)]
struct RenderIndex<'a> {
    #[serde(flatten)]
    layout: RenderLayout<'a>,
    #[serde(flatten)]
    general_render: RenderMessages<'a>,
    #[serde(flatten)]
    routes_render: RenderRoutes<'a>,
    #[serde(flatten)]
    buttons_render: RenderButtons<'a>,
    // Devices.
    devices: &'a Devices,
    // Hazards.
    //hazards: &'a [HazardData],
}

impl<'a> RenderIndex<'a> {
    fn new(devices: &'a Devices) -> Self {
        Self {
            layout: RenderLayout::new(),
            general_render: RenderMessages::new(),
            routes_render: RenderRoutes::new(),
            buttons_render: RenderButtons::new(),
            devices,
        }
    }
}

pub(crate) async fn index(State(state): State<AppState>) -> Result<Html<String>, Error> {
    let template = error_with_info(
        &state.env,
        state.env.get_template("index"),
        &t!("templates_error.get_index_template"),
    )?;

    // TODO: Only the hazards associated with each discovered device must be considered.
    //let all_hazards = retrieve_all_hazards();

    let devices = state.devices.lock().await;

    let rendered = error_with_info(
        &state.env,
        template.render(RenderIndex::new(&devices)),
        &t!("templates_error.render_index_template"),
    )?;

    Ok(Html(rendered))
}
