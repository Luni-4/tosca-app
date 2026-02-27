use std::{borrow::Cow, collections::HashMap};

use serde::Serialize;

#[derive(Default, Serialize)]
pub(crate) enum LightMode {
    #[default]
    Manual,
    MotionDetection,
    AmbientLight,
}

#[derive(Serialize)]
pub(crate) struct StateRoutes {
    manual_route: &'static str,
    manual_mode: &'static str,
    manual: Cow<'static, str>,
    motion_detection_route: &'static str,
    motion_detection_mode: &'static str,
    motion_detection: Cow<'static, str>,
    ambient_light_route: &'static str,
    ambient_light_mode: &'static str,
    ambient_light: Cow<'static, str>,
}

impl StateRoutes {
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            manual_route: "/manual",
            manual_mode: "manual",
            manual: t!("light.manual"),
            motion_detection_route: "/motion-detection",
            motion_detection_mode: "motion",
            motion_detection: t!("light.motion_detection"),
            ambient_light_route: "/ambient-light",
            ambient_light_mode: "ambient",
            ambient_light: t!("light.ambient_light"),
        }
    }
}

#[derive(Serialize)]
pub(crate) struct RouteMetadata {
    name: Cow<'static, str>,
    description: Cow<'static, str>,
}

impl RouteMetadata {
    #[inline]
    pub(crate) fn new(name: Cow<'static, str>, description: Cow<'static, str>) -> Self {
        Self { name, description }
    }
}

#[derive(Serialize)]
pub(crate) struct LocalizedHazard {
    id: u16,
    name: Cow<'static, str>,
    description: Cow<'static, str>,
    category_name: &'static str,
    // TODO: Use a state structure to record the disabled value in order to
    // present it to the UI
    is_disabled: bool,
}

impl LocalizedHazard {
    pub(crate) fn new(id: u16, category_name: &'static str) -> LocalizedHazard {
        Self {
            id,
            name: t!(format!("hazards_{}.name", id)),
            description: t!(format!("hazards_{}.description", id)),
            category_name,
            is_disabled: false,
        }
    }

    pub(crate) const fn id(&self) -> u16 {
        self.id
    }

    pub(crate) const fn set_disabled(&mut self, disabled: bool) {
        self.is_disabled = disabled;
    }
}

#[derive(Serialize)]
pub(crate) struct RouteData {
    id: usize,
    hazards: Vec<LocalizedHazard>,
    is_disabled: bool,
}

impl RouteData {
    pub(crate) fn new(id: usize, hazards: Vec<LocalizedHazard>) -> Self {
        Self {
            id,
            hazards,
            is_disabled: false,
        }
    }

    pub(crate) const fn set_disabled(&mut self, disabled: bool) {
        self.is_disabled = disabled;
    }
}

#[derive(Serialize)]
pub(crate) struct Route {
    metadata: RouteMetadata,
    data: RouteData,
}

impl Route {
    pub(crate) fn new(metadata: RouteMetadata, data: RouteData) -> Self {
        Self { metadata, data }
    }
}

#[derive(Serialize)]
pub(crate) struct DemoLightInfo {
    title_description: Cow<'static, str>,
    description: Cow<'static, str>,
    modes: Cow<'static, str>,
    state_routes: StateRoutes,
    commands: Cow<'static, str>,
    route_title_description: Cow<'static, str>,
    hazard_title_description: Cow<'static, str>,
    routes: HashMap<String, Route>,
}

impl DemoLightInfo {
    #[inline]
    pub(crate) fn new(routes: HashMap<String, Route>) -> Self {
        Self {
            title_description: t!("device.title_description"),
            description: t!("light.description"),
            modes: t!("light.modes"),
            state_routes: StateRoutes::new(),
            commands: t!("light.commands"),
            route_title_description: t!("device.route_title_description"),
            hazard_title_description: t!("device.hazard_title_description"),
            routes,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct DemoLight {
    id: usize,
    #[serde(flatten)]
    info: DemoLightInfo,
    mode: LightMode,
    has_events: bool,
}

impl DemoLight {
    #[inline]
    pub(crate) fn new(id: usize, routes: HashMap<String, Route>) -> Self {
        Self {
            id,
            info: DemoLightInfo::new(routes),
            mode: LightMode::default(),
            has_events: false,
        }
    }

    #[inline]
    pub(crate) fn with_events(id: usize, routes: HashMap<String, Route>) -> Self {
        Self {
            id,
            info: DemoLightInfo::new(routes),
            mode: LightMode::default(),
            has_events: true,
        }
    }

    #[inline]
    pub(crate) fn change_state(&mut self, state: &str) {
        self.mode = match state {
            "motion" => LightMode::MotionDetection,
            "ambient" => LightMode::AmbientLight,
            // If no valid mode is passed, the default mode will be used.
            _ => LightMode::default(),
        };
    }

    #[inline]
    pub(crate) fn is_state_route(route: &str) -> bool {
        route == "/manual" || route == "/motion-detection" || route == "/ambient-light"
    }

    pub(crate) fn update_policy_flags<F>(&mut self, mut is_hazard_disabled: F)
    where
        F: FnMut(u16, &str) -> bool,
    {
        for route in self.info.routes.values_mut() {
            for hazard in &mut route.data.hazards {
                let disabled = is_hazard_disabled(hazard.id(), hazard.category_name);
                hazard.set_disabled(disabled);
            }

            let route_disabled = route.data.hazards.iter().any(|hazard| hazard.is_disabled);
            route.data.set_disabled(route_disabled);
        }
    }
}

#[derive(Serialize)]
pub(crate) struct Devices(pub(crate) Vec<DemoLight>);

impl Devices {
    pub(crate) const fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    #[inline]
    pub(crate) fn add_device(&mut self, light: DemoLight) {
        self.0.push(light);
    }
}
