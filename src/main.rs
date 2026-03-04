#[macro_use]
extern crate rust_i18n;

mod config;
mod devices;
mod discovery;
mod error;
mod events;
mod index;
mod layout;
#[cfg(feature = "logging")]
mod logging;
mod privacy;
mod privacy_policy;
mod request;
mod utils;

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use tosca::events::Events;

use tosca_controller::controller::Controller;

use axum::{
    Router,
    handler::HandlerWithoutStateExt,
    routing::{get, post},
};

use minijinja::{Environment, value::Value};

use tokio::sync::Mutex;
use tokio::sync::broadcast::Receiver;

use tower_http::services::ServeDir;

use crate::config::Configuration;
use crate::devices::Devices;
use crate::discovery::run_discovery;
use crate::error::{missing_assets, missing_route};
use crate::events::event_stream;
use crate::index::index;
use crate::layout::Layout;
use crate::privacy::privacy;
use crate::privacy_policy::{PrivacyPolicyState, toggle_category, toggle_hazard};
use crate::request::send_request;
use crate::utils::{add_functions_to_env, create_controller};

// Load all available translations.
//
// Use `en` as locale fallback in case of missing translations.
rust_i18n::i18n!("locales", fallback = "en");

macro_rules! builtin_templates {
    ($(($name:expr, $template:expr)),+) => {
        [
            $(
                (
                    $name,
                    include_str!(concat!(env!("CARGO_MANIFEST_DIR"),"/templates/", $template)),
                )
            ),+
        ]
    }
}

static TEMPLATES: &[(&str, &str)] = &builtin_templates![
    // Layout page.
    ("layout", "layout.html"),
    ("head", "head.html"),
    ("navbar", "navbar.html"),
    ("footer", "footer.html"),
    // Index page.
    ("index", "index.html"),
    ("devices", "devices.html"),
    // Privacy page.
    ("privacy", "privacy.html"),
    // Error page.
    ("error", "error.html"),
    // Widgets.
    ("spinner", "widgets/spinner.html")
];

#[derive(Clone)]
struct AppState {
    env: Arc<Environment<'static>>,
    controller: Arc<Mutex<Controller>>,
    devices: Arc<Mutex<Devices>>,
    // TODO: Use a std::Mutex because we are dealing with data only.
    devices_receivers: Arc<Mutex<HashMap<usize, Receiver<Events>>>>,
    policy_state: Arc<Mutex<PrivacyPolicyState>>,
}

impl AppState {
    fn new(env: Environment<'static>, controller: Controller) -> Self {
        Self {
            env: Arc::new(env),
            controller: Arc::new(Mutex::new(controller)),
            devices: Arc::new(Mutex::new(Devices::new())),
            devices_receivers: Arc::new(Mutex::new(HashMap::new())),
            policy_state: Arc::new(Mutex::new(PrivacyPolicyState::default())),
        }
    }

    fn controller_clone(&self) -> Arc<Mutex<Controller>> {
        self.controller.clone()
    }
}

#[tokio::main]
async fn main() {
    // Retrieve configuration data.
    let config = Configuration::load();

    // Set locale language.
    let lang = config.language.as_str();
    rust_i18n::set_locale(lang);

    // Initialize subscriber.
    #[cfg(feature = "logging")]
    logging::create_subscriber();

    let mut env = Environment::new();

    for (name, src) in TEMPLATES {
        env.add_template(name, src)
            .expect("Built-in template internal failure.");
    }

    // Add global functions to minijinja environment.
    add_functions_to_env(&mut env);

    // Add global variable
    env.add_global("app", Value::from_serialize(Layout::new(lang)));

    // Create controller.
    let controller = create_controller();

    // Pass template environment and controller via state
    let app_state = AppState::new(env, controller);
    // Clone the controller reference
    let controller_clone = app_state.controller_clone();

    // Loads the directory containing assets such as `CSS` or `JS` files.
    let serve_dir = ServeDir::new("assets").not_found_service(missing_assets.into_service());

    // Define routes
    let app = Router::new()
        .route("/", get(index))
        .route("/privacy", get(privacy))
        .route("/privacy/category", post(toggle_category))
        .route("/privacy/hazard", post(toggle_hazard))
        .route("/events/{device_id}", get(event_stream))
        .route("/discovery", post(run_discovery))
        .route("/request", post(send_request))
        .nest_service("/assets/", serve_dir.clone())
        .fallback_service(serve_dir)
        .fallback(missing_route)
        .with_state(app_state);

    // Creates the web controller listener bind.
    let listener_bind = SocketAddr::new(IpAddr::V4(config.host), config.port);

    // Creates listener.
    let listener = tokio::net::TcpListener::bind(&listener_bind)
        .await
        .expect("Listener creation failed.");

    // Prints listener bind and controller startup message.
    #[cfg(feature = "logging")]
    {
        // Navbar route.
        tracing::info!(r#"Home Page: [GET, "/"]"#);
        tracing::info!(r#"Privacy Page: [GET, "/privacy"]"#);

        // Device controller commands.
        tracing::info!(r#"Discovery Route: [POST, "/discovery"]"#);
        tracing::info!(r#"Request Route: [POST, "/request"]"#);

        // Assets service.
        tracing::info!(r#"Assets Service: [SERVICE, "/assets"]"#);

        // Server information.
        tracing::info!("Tosca app reachable at this address: {listener_bind}");
        tracing::info!("Starting tosca app...");
    }

    // Runs server.
    axum::serve(listener, app)
        .await
        .expect("Server startup failed.");

    if let Some(controller) = Arc::into_inner(controller_clone) {
        controller.into_inner().shutdown().await;
    }
}
