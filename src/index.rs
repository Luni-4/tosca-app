use std::borrow::Cow;

use axum::extract::State;
use axum::response::Html;

use serde::Serialize;

use crate::AppState;
use crate::devices::Devices;
use crate::error::{Error, error_with_info};
use crate::layout::INDEX_LINK;
use crate::utils::parse_category;

#[derive(Serialize)]
struct RenderMessages {
    no_devices_message: Cow<'static, str>,
    spinner_text: Cow<'static, str>,
}

impl RenderMessages {
    #[inline]
    fn new() -> Self {
        Self {
            no_devices_message: t!("labels.no_devices"),
            spinner_text: t!("labels.spinner_text"),
        }
    }
}

#[derive(Serialize)]
struct RenderButtons {
    discovery_route: &'static str,
    discovery_button: Cow<'static, str>,
}

impl RenderButtons {
    #[inline]
    fn new() -> Self {
        Self {
            discovery_route: "/discovery",
            discovery_button: t!("buttons.discovery"),
        }
    }
}

#[derive(Serialize)]
struct RenderIndex<'a> {
    nav_link_selected: &'static str,
    #[serde(flatten)]
    general_render: RenderMessages,
    #[serde(flatten)]
    buttons_render: RenderButtons,
    // Devices.
    devices: &'a Devices,
}

impl<'a> RenderIndex<'a> {
    fn new(devices: &'a Devices) -> Self {
        Self {
            nav_link_selected: INDEX_LINK,
            general_render: RenderMessages::new(),
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

    let mut devices = state.devices.lock().await;

    {
        let policy_state = state.policy_state.lock().await;

        for device in &mut devices.0 {
            device.update_policy_flags(|hazard_id, category_name| {
                if let Some(category) = parse_category(category_name)
                    && policy_state.is_category_blocked(category)
                {
                    return true;
                }

                policy_state.is_hazard_blocked(hazard_id)
            });
        }
    }

    let rendered = error_with_info(
        &state.env,
        template.render(RenderIndex::new(&devices)),
        &t!("templates_errors.render_index_template"),
    )?;

    Ok(Html(rendered))
}
