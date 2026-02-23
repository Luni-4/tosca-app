use std::borrow::Cow;

use axum::{
    extract::{Json, State},
    http::{StatusCode, Uri},
    response::{Html, IntoResponse, Response},
};

use minijinja::Environment;

use serde::Serialize;

use crate::AppState;

const EMAIL_ERROR: &str = "<a href=\"mailto:error@toscalabs.org\">error@toscalabs.org</a>";

#[derive(Serialize)]
struct RenderError<'a> {
    error_status: u16,
    title: Cow<'static, str>,
    email: Cow<'static, str>,
    copy_button_alt: Cow<'static, str>,
    description: &'a str,
    info: Option<String>,
    index: &'static str,
    goto_message: Cow<'static, str>,
}

impl<'a> RenderError<'a> {
    fn description(description: &'a str) -> Self {
        Self::new(description, None, 500)
    }

    fn page_not_found(description: &'a str) -> Self {
        Self::new(description, None, 404)
    }

    fn error(description: &'a str, info: String) -> Self {
        Self::new(description, Some(info), 500)
    }

    fn new(description: &'a str, info: Option<String>, error_status: u16) -> Self {
        Self {
            error_status,
            title: t!("error.title", status = error_status),
            email: t!("error.report_message", email = EMAIL_ERROR),
            copy_button_alt: t!("error.copy_button_alt"),
            description,
            info,
            index: "/",
            goto_message: t!("error.goto_home"),
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
        tracing::error!("{}", t!("error.request_error", error = e));
        Error::error_page(env, description, e)
    })
}

pub(crate) async fn missing_assets() -> Error {
    #[cfg(feature = "logging")]
    tracing::error!("{}", t!("error.assets_error"));
    Error::new(ErrorState::Assets, t!("error.assets_error"), true)
}

pub(crate) async fn missing_route(State(state): State<AppState>, uri: Uri) -> Error {
    let error = t!("error.missing_route", route = uri);
    #[cfg(feature = "logging")]
    tracing::error!("{error}");
    Error::render_template(&state.env, RenderError::page_not_found(&error))
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
    is_error_500: bool,
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
        let is_error_500 = context.error_status == 500;

        let template = match env.get_template("error") {
            Ok(template) => template,
            Err(e) => return Self::new(ErrorState::Template, e.to_string(), true),
        };

        let rendered = match template.render(context) {
            Ok(rendered) => rendered,
            Err(e) => return Self::new(ErrorState::Render, e.to_string(), true),
        };

        Self::new(ErrorState::Success, rendered, is_error_500)
    }

    fn new(state: ErrorState, data: impl Into<Cow<'static, str>>, is_error_500: bool) -> Self {
        Self {
            state,
            data: data.into(),
            is_error_500,
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

        let status_code = if self.is_error_500 {
            StatusCode::INTERNAL_SERVER_ERROR
        } else {
            StatusCode::NOT_FOUND
        };

        (status_code, body).into_response()
    }
}
