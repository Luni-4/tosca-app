use std::borrow::Cow;

use chrono::{Datelike, Utc};

use serde::Serialize;

const WEB_APP_TITLE: &str = "Tosca App";

pub(crate) fn footer() -> String {
    format!("{} ToscaLabs. {}", Utc::now().year(), t!("footer.rights"))
}

#[derive(Serialize)]
pub(crate) struct RenderLayout<'a> {
    title: &'static str,
    github_link: &'static str,
    github_description: Cow<'a, str>,
    footer: String,
}

impl RenderLayout<'_> {
    pub(crate) fn new() -> Self {
        Self {
            title: WEB_APP_TITLE,
            github_link: "https://github.com/ToscaLabs",
            github_description: t!("navbar.github_description"),
            footer: footer(),
        }
    }
}
