use std::collections::HashSet;

use axum::extract::State;
use axum::http::StatusCode;
use axum_extra::extract::Form;

use serde::Deserialize;

use tosca::hazards::{Category, Hazard, Hazards};
use tosca_controller::policy::Policy;

use crate::AppState;
use crate::error::Error;
use crate::utils::parse_category;

#[derive(Default, Clone)]
pub(crate) struct PrivacyPolicyState {
    blocked_categories: HashSet<Category>,
    blocked_hazards: HashSet<u16>,
}

impl PrivacyPolicyState {
    #[inline]
    pub(crate) fn is_category_blocked(&self, category: Category) -> bool {
        self.blocked_categories.contains(&category)
    }

    #[inline]
    pub(crate) fn is_hazard_blocked(&self, hazard_id: u16) -> bool {
        self.blocked_hazards.contains(&hazard_id)
    }

    #[inline]
    pub(crate) fn set_category_blocked(&mut self, category: Category, blocked: bool) {
        if blocked {
            self.blocked_categories.insert(category);
        } else {
            self.blocked_categories.remove(&category);
        }
    }

    #[inline]
    pub(crate) fn set_hazard_blocked(&mut self, hazard_id: u16, blocked: bool) {
        if blocked {
            self.blocked_hazards.insert(hazard_id);
        } else {
            self.blocked_hazards.remove(&hazard_id);
        }
    }

    pub(crate) fn build_policy(&self) -> Policy {
        let mut global_hazards = Hazards::new();

        for category in &self.blocked_categories {
            for hazard in category.hazards() {
                global_hazards.add(*hazard);
            }
        }

        for &id in &self.blocked_hazards {
            if let Some(h) = Hazard::from_id(id) {
                global_hazards.add(h);
            }
        }

        Policy::new(global_hazards)
    }
}

#[derive(Deserialize)]
pub(crate) struct ToggleCategoryForm {
    category: String,
    blocked: bool,
}

#[derive(Deserialize)]
pub(crate) struct ToggleHazardForm {
    hazard_id: u16,
    blocked: bool,
}

pub(crate) async fn toggle_category(
    State(state): State<AppState>,
    Form(payload): Form<ToggleCategoryForm>,
) -> Result<StatusCode, Error> {
    let env = state.env.clone();

    let category = parse_category(&payload.category).ok_or_else(|| {
        Error::description_page(
            &env,
            &t!(
                "privacy_policy_errors.invalid_category",
                category = payload.category
            ),
        )
    })?;

    {
        let mut policy_state = state.policy_state.lock().await;
        policy_state.set_category_blocked(category, payload.blocked);

        let policy = policy_state.build_policy();

        let mut controller = state.controller.lock().await;
        controller.change_policy(policy);
    }

    Ok(StatusCode::NO_CONTENT)
}

pub(crate) async fn toggle_hazard(
    State(state): State<AppState>,
    Form(payload): Form<ToggleHazardForm>,
) -> Result<StatusCode, Error> {
    let env = state.env.clone();

    if Hazard::from_id(payload.hazard_id).is_none() {
        return Err(Error::description_page(
            &env,
            &t!(
                "privacy_policy_errors.invalid_hazard_id",
                id = payload.hazard_id
            ),
        ));
    }

    {
        let mut policy_state = state.policy_state.lock().await;
        policy_state.set_hazard_blocked(payload.hazard_id, payload.blocked);

        let policy = policy_state.build_policy();

        let mut controller = state.controller.lock().await;
        controller.change_policy(policy);
    }

    Ok(StatusCode::NO_CONTENT)
}
