use std::borrow::Cow;

use chrono::{Datelike, Utc};

use serde::Serialize;

const WEB_APP_TITLE: &str = "Tosca App";

pub(crate) const INDEX_LINK: &str = "/";
pub(crate) const PRIVACY_LINK: &str = "/privacy";

#[derive(Serialize)]
struct Item {
    link: &'static str,
    name: Cow<'static, str>,
}

impl Item {
    fn new(link: &'static str, name: Cow<'static, str>) -> Self {
        Self { link, name }
    }
}

pub(crate) fn footer() -> String {
    format!("{} ToscaLabs. {}", Utc::now().year(), t!("footer.rights"))
}

#[derive(Serialize)]
pub(crate) struct Layout {
    lang: &'static str,
    title: &'static str,
    navbar_items: [Item; 2],
    github_description: Cow<'static, str>,
    footer: String,
}

impl Layout {
    pub(crate) fn new(lang: &'static str) -> Self {
        Self {
            lang,
            title: WEB_APP_TITLE,
            navbar_items: [
                Item::new(INDEX_LINK, t!("navbar.home_name")),
                Item::new(PRIVACY_LINK, t!("navbar.privacy_name")),
            ],
            github_description: t!("navbar.github_description"),
            footer: footer(),
        }
    }
}
