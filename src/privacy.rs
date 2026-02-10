use std::borrow::Cow;

use axum::extract::State;
use axum::response::Html;

use serde::Serialize;

use tosca::hazards::ALL_CATEGORIES;

use crate::AppState;
use crate::error::{Error, error_with_info};
use crate::layout::PRIVACY_LINK;

#[derive(Serialize)]
struct CategoryData {
    id: &'static str,
    name: Cow<'static, str>,
    description: Cow<'static, str>,
}

impl CategoryData {
    const fn new(
        id: &'static str,
        name: Cow<'static, str>,
        description: Cow<'static, str>,
    ) -> Self {
        Self {
            id,
            name,
            description,
        }
    }
}

#[derive(Serialize)]
struct HazardsCard {
    title: Cow<'static, str>,
    description_title: Cow<'static, str>,
    description: Cow<'static, str>,
    categories: Vec<CategoryData>,
}

impl HazardsCard {
    fn new() -> Self {
        Self {
            title: t!("privacy_hazards_card.title"),
            description_title: t!("privacy_hazards_card.description_title"),
            description: t!("privacy_hazards_card.description"),
            categories: ALL_CATEGORIES
                .iter()
                .map(|category| {
                    let category_name = category.name();
                    CategoryData::new(
                        category_name,
                        t!(format!("hazard_categories.{}", category_name)),
                        t!(format!("hazard_categories_{}.description", category_name)),
                    )
                })
                .collect(),
        }
    }
}

#[derive(Serialize)]
pub(crate) struct Privacy {
    nav_link_selected: &'static str,
    hazards_card: HazardsCard,
}

impl Privacy {
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            nav_link_selected: PRIVACY_LINK,
            hazards_card: HazardsCard::new(),
        }
    }
}

pub(crate) async fn privacy(State(state): State<AppState>) -> Result<Html<String>, Error> {
    let template = error_with_info(
        &state.env,
        state.env.get_template("privacy"),
        &t!("templates_error.get_privacy_template"),
    )?;

    let rendered = error_with_info(
        &state.env,
        template.render(Privacy::new()),
        &t!("templates_error.render_privacy_template"),
    )?;

    Ok(Html(rendered))
}
