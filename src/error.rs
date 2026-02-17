use std::borrow::Cow;

use axum::{
    extract::{Json, State},
    http::{StatusCode, Uri},
    response::{Html, IntoResponse, Response},
};

use minijinja::Environment;

use serde::Serialize;

use crate::AppState;

#[derive(Serialize)]
struct RenderError<'a> {
    description: &'a str,
    info: Option<String>,
    goto_message: Cow<'static, str>,
    index: &'static str,
}

impl<'a> RenderError<'a> {
    fn description(description: &'a str) -> Self {
        Self::new(description, None)
    }

    fn error(description: &'a str, info: String) -> Self {
        Self::new(description, Some(info))
    }

    fn new(description: &'a str, info: Option<String>) -> Self {
        Self {
            description,
            info,
            goto_message: t!("labels.go_to_devices"),
            index: "/",
        }
    }
}

pub(crate) fn error_with_info<T, E: std::error::Error>(
    env: &Environment<'static>,
    res: Result<T, E>,
    description: &str,
) -> Result<T, Error> {
    res.map_err(|e| {
        #[cfg(feature = "logging")]
        tracing::error!("{} ~> {e}", t!("logging.request_error"));
        Error::error_page(env, description, e)
    })
}

pub(crate) async fn missing_assets() -> Error {
    #[cfg(feature = "logging")]
    tracing::error!(
        "{} ~> {}",
        t!("logging.request_error"),
        t!("startup_errors.assets")
    );
    Error::new(ErrorState::Assets, t!("startup_errors.assets").into())
}

pub(crate) async fn missing_route(State(state): State<AppState>, uri: Uri) -> Error {
    let error = format!("{} `{uri}`", t!("labels.missing_route"));
    #[cfg(feature = "logging")]
    tracing::error!("{} ~> {error}", t!("logging.request_error"));
    Error::description_page(&state.env, &error)
}

#[derive(Serialize)]
struct JsonError<'a> {
    // Error description.
    description: &'a str,
    // Information about an error.
    #[serde(skip_serializing_if = "Option::is_none")]
    info: Option<Cow<'static, str>>,
}

impl<'a> JsonError<'a> {
    fn with_description(description: &'a str) -> Self {
        Self {
            description,
            info: None,
        }
    }

    fn with_description_error(description: &'a str, info: Cow<'static, str>) -> Self {
        Self {
            description,
            info: Some(info),
        }
    }
}

enum ErrorState {
    Success,
    Assets,
    Template,
    Render,
}

pub(crate) struct Error {
    state: ErrorState,
    data: Cow<'static, str>,
}

impl Error {
    pub(crate) fn description_page(env: &Environment<'static>, description: &str) -> Self {
        Self::render_template(env, RenderError::description(description))
    }

    fn error_page(
        env: &Environment<'static>,
        description: &str,
        info: impl std::error::Error,
    ) -> Self {
        Self::render_template(env, RenderError::error(description, info.to_string()))
    }

    fn render_template(env: &Environment<'static>, context: RenderError) -> Self {
        let template = match env.get_template("error") {
            Ok(template) => template,
            Err(e) => return Self::new(ErrorState::Template, e.to_string()),
        };

        let rendered = match template.render(context) {
            Ok(rendered) => rendered,
            Err(e) => return Self::new(ErrorState::Render, e.to_string()),
        };

        Self::new(ErrorState::Success, rendered)
    }

    fn new(state: ErrorState, data: impl Into<Cow<'static, str>>) -> Self {
        Self {
            state,
            data: data.into(),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let body = match self.state {
            ErrorState::Success => Html(self.data).into_response(),
            ErrorState::Assets => Json(JsonError::with_description(&self.data)).into_response(),
            ErrorState::Template => Json(JsonError::with_description_error(
                &t!("templates_error.get_error_template"),
                self.data,
            ))
            .into_response(),
            ErrorState::Render => Json(JsonError::with_description_error(
                &t!("templates_error.render_error_template"),
                self.data,
            ))
            .into_response(),
        };

        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}
