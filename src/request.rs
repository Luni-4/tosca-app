use tosca::parameters::{ParameterKind, ParametersValues};

use tosca_controller::response::Response;

use axum::extract::State;
use axum::response::Redirect;

use axum_extra::extract::Form;

use minijinja::Environment;

use serde::Deserialize;
use serde_json::Value;

use crate::AppState;
use crate::error::{Error, error_with_info};

#[derive(Deserialize)]
#[cfg_attr(feature = "logging", derive(Debug))]
pub(crate) struct Request {
    device_id: usize,
    #[serde(default)]
    device_state: String,
    route: String,
    #[serde(default)]
    ids: Vec<ParameterKind>,
    #[serde(default)]
    names: Vec<String>,
    #[serde(default)]
    values: Vec<String>,
}

fn create_parameters<'a>(
    env: &Environment<'static>,
    ids: &[ParameterKind],
    names: &'a [String],
    values: Vec<String>,
) -> Result<ParametersValues<'a>, Error> {
    let mut parameters = ParametersValues::new();
    for (i, (value, id)) in values.into_iter().zip(ids).enumerate() {
        match id {
            ParameterKind::Bool { .. } => {
                parameters.bool(&names[i], value.is_empty());
            }
            ParameterKind::U8 { .. } => {
                let value =
                    error_with_info(env, value.parse(), &t!("parsing_errors.u8_parameter"))?;
                parameters.u8(&names[i], value);
            }
            ParameterKind::U16 { .. } => {
                let value =
                    error_with_info(env, value.parse(), &t!("parsing_errors.u16_parameter"))?;
                parameters.u16(&names[i], value);
            }
            ParameterKind::U32 { .. } | ParameterKind::RangeU32 { .. } => {
                let value =
                    error_with_info(env, value.parse(), &t!("parsing_errors.u32_parameter"))?;
                parameters.u32(&names[i], value);
            }
            ParameterKind::U64 { .. } | ParameterKind::RangeU64 { .. } => {
                let value =
                    error_with_info(env, value.parse(), &t!("parsing_errors.u64_parameter"))?;
                parameters.u64(&names[i], value);
            }
            ParameterKind::F32 { .. } => {
                let value =
                    error_with_info(env, value.parse(), &t!("parsing_errors.f32_parameter"))?;
                parameters.f32(&names[i], value);
            }
            ParameterKind::F64 { .. } | ParameterKind::RangeF64 { .. } => {
                let value =
                    error_with_info(env, value.parse(), &t!("parsing_errors.f64_parameter"))?;
                parameters.f64(&names[i], value);
            }
            ParameterKind::CharsSequence { .. } => {
                parameters.characters_sequence(&names[i], value);
            }
        }
    }
    Ok(parameters)
}

pub(crate) async fn send_request(
    State(state): State<AppState>,
    Form(request): Form<Request>,
) -> Result<Redirect, Error> {
    #[cfg(feature = "logging")]
    tracing::info!("{:?}", request);

    let env = state.env;
    let Request {
        device_id,
        device_state,
        route,
        ids,
        names,
        values,
    } = request;

    // Send a request to a controller and obtain a response.
    let response = {
        let controller = state.controller.lock().await;

        // Find device sender.
        let device_sender = error_with_info(
            &env,
            controller.device(device_id),
            &t!("request_errors.not_found_device"),
        )?;

        // Send request.
        let request_sender = error_with_info(
            &env,
            device_sender.request(&route),
            &t!("request_errors.request_creation"),
        )?;

        // Obtain response.
        if ids.is_empty() {
            // Send request.
            error_with_info(
                &env,
                request_sender.send().await,
                &t!("request_errors.send_request_with_default_params"),
            )?
        } else {
            // Create parameters.
            let parameters = create_parameters(&env, &ids, &names, values)?;
            // Send request with parameters.
            error_with_info(
                &env,
                request_sender.send_with_parameters(&parameters).await,
                &t!("request_errors.send_request_with_params"),
            )?
        }
    };

    // TODO: Add responses to response log.
    //
    // Check response kind.
    match response {
        Response::OkBody(response) => {
            error_with_info(
                &env,
                response.parse_body().await,
                &t!("parsing_errors.ok_response"),
            )?;
        }
        Response::SerialBody(response) => {
            error_with_info(
                &env,
                response.parse_body::<Value>().await,
                &t!("parsing_errors.serial_response"),
            )?;
        }
        Response::InfoBody(_response) => {}
        // TODO: How to treat a skip response because of privacy here. Add to
        // response log.
        Response::Skipped => todo!("Add skipped response to response log"),
        Response::StreamBody(_) => todo!("Stream response not implemented"),
    }

    // If a state has been assigned, change the state.
    if !device_state.is_empty() {
        let mut devices = state.devices.lock().await;

        let device = devices.0.get_mut(device_id).ok_or(Error::description_page(
            &env,
            &t!("request_errors.not_found_device"),
        ))?;

        device.change_state(&device_state);
    }

    // Redirect to index
    Ok(Redirect::to("/"))
}
