use std::borrow::Cow;

use axum::extract::State;
use axum::response::Html;
use serde::Serialize;

use tosca::hazards::{ALL_CATEGORIES, Category};

use crate::AppState;
use crate::error::{Error, error_with_info};
use crate::layout::PRIVACY_LINK;

#[derive(Serialize)]
struct HazardData {
    id: u16,
    name: Cow<'static, str>,
    description: Cow<'static, str>,
    is_disabled: bool,
}

impl HazardData {
    fn new(id: u16, is_disabled: bool) -> Self {
        Self {
            id,
            name: t!(format!("hazards_{}.name", id)),
            description: t!(format!("hazards_{}.description", id)),
            is_disabled,
        }
    }
}

#[derive(Serialize)]
struct CategoryCard {
    id: &'static str,
    name: Cow<'static, str>,
    description: Cow<'static, str>,
    hazards_label: Cow<'static, str>,
    block_all_label: Cow<'static, str>,
    is_disabled: bool,
    hazards: Vec<HazardData>,
}

impl CategoryCard {
    fn new(
        category: Category,
        is_category_disabled: impl Fn(Category) -> bool,
        is_hazard_disabled: impl Fn(u16) -> bool,
    ) -> Self {
        let id = category.name();
        let category_disabled = is_category_disabled(category);

        let mut hazards = Vec::new();
        for hazard in category.hazards() {
            let mut disabled = is_hazard_disabled(hazard.id());

            // If category is disabled, all hazards appear disabled and are non-clickable
            if category_disabled {
                disabled = true;
            }

            hazards.push(HazardData::new(hazard.id(), disabled));
        }

        hazards.sort_by_key(|hazard| hazard.id);

        Self {
            id,
            name: t!(format!("hazard_categories.{}", id)),
            description: t!(format!("hazard_categories_{}.description", id)),
            hazards_label: t!("privacy_hazards_card.hazards_label"),
            block_all_label: t!("privacy_hazards_card.block_all_label"),
            is_disabled: category_disabled,
            hazards,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct Privacy {
    nav_link_selected: &'static str,
    categories: Vec<CategoryCard>,
}

impl Privacy {
    pub(crate) fn new(
        is_category_disabled: impl Fn(Category) -> bool,
        is_hazard_disabled: impl Fn(u16) -> bool,
    ) -> Self {
        Self {
            nav_link_selected: PRIVACY_LINK,
            categories: ALL_CATEGORIES
                .iter()
                .map(|c| CategoryCard::new(*c, &is_category_disabled, &is_hazard_disabled))
                .collect(),
        }
    }
}

pub(crate) async fn privacy(State(state): State<AppState>) -> Result<Html<String>, Error> {
    let template = error_with_info(
        &state.env,
        state.env.get_template("privacy"),
        &t!("templates_error.get_privacy_template"),
    )?;

    let privacy = {
        let policy_state = state.policy_state.lock().await;
        Privacy::new(
            |category| policy_state.is_category_blocked(category),
            |id| policy_state.is_hazard_blocked(id),
        )
    };

    let rendered = error_with_info(
        &state.env,
        template.render(privacy),
        &t!("templates_error.render_privacy_template"),
    )?;

    Ok(Html(rendered))
}
