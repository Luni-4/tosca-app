use std::time::Duration;

use tosca::hazards::{Category, Hazard};

use tosca_controller::controller::Controller;
use tosca_controller::discovery::{Discovery, TransportProtocol};

use minijinja::Environment;
use minijinja::value::ViaDeserialize;

// Default duration for the discovery process.
const DEFAULT_DISCOVERY_DURATION: Duration = Duration::from_secs(1);

pub(crate) fn create_controller() -> Controller {
    // Create discovery searcher
    let discovery = Discovery::new("tosca")
        .timeout(DEFAULT_DISCOVERY_DURATION)
        .transport_protocol(TransportProtocol::UDP)
        .disable_ipv6()
        .disable_network_interface("docker0");

    Controller::new(discovery)
}

pub(crate) fn add_functions_to_env(env: &mut Environment<'_>) {
    env.add_function("hazard_id", |hazard: ViaDeserialize<Hazard>| {
        hazard.data().id
    });
    env.add_function("hazard_name", |hazard: ViaDeserialize<Hazard>| {
        hazard.data().name
    });
    env.add_function("hazard_category", |hazard: ViaDeserialize<Hazard>| {
        hazard.data().category_name
    });
}

#[inline]
pub(crate) fn parse_category(name: &str) -> Option<Category> {
    match name.trim().to_ascii_lowercase().as_str() {
        "safety" => Some(Category::Safety),
        "privacy" => Some(Category::Privacy),
        "financial" => Some(Category::Financial),
        _ => None,
    }
}
