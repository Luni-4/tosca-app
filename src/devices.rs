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
pub(crate) struct LocalizedHazard {
    id: u16,
    name: Cow<'static, str>,
    description: Cow<'static, str>,
    category_name: Cow<'static, str>,
    category_description: Cow<'static, str>,
}

impl LocalizedHazard {
    pub(crate) fn new(id: u16, category_name: &'static str) -> LocalizedHazard {
        Self {
            id,
            name: t!(format!("hazards_{}.name", id)),
            description: t!(format!("hazards_{}.description", id)),
            category_name: t!(format!("hazard_categories.{}", category_name)),
            category_description: t!(format!("hazard_categories_{}.description", category_name)),
        }
    }
}

#[derive(Serialize)]
pub(crate) struct DemoLightInfo {
    modes: String,
    manual_route: String,
    manual: String,
    manual_mode: String,
    motion_detection_route: String,
    motion_detection_mode: String,
    motion_detection: String,
    ambient_light_route: String,
    ambient_light_mode: String,
    ambient_light: String,
    commands: String,
    on_route: String,
    on: String,
    off_route: String,
    off: String,
    toggle_route: String,
    toggle: String,
    hazards: HashMap<String, Vec<LocalizedHazard>>,
}

impl DemoLightInfo {
    #[inline]
    pub(crate) fn new(hazards: HashMap<String, Vec<LocalizedHazard>>) -> Self {
        Self {
            modes: t!("light.modes").into_owned(),
            manual_route: t!("light.manual_route").into_owned(),
            manual_mode: t!("light.manual_mode").into_owned(),
            manual: t!("light.manual").into_owned(),
            motion_detection_route: t!("light.motion_detection_route").into_owned(),
            motion_detection_mode: t!("light.motion_detection_mode").into_owned(),
            motion_detection: t!("light.motion_detection").into_owned(),
            ambient_light_route: t!("light.ambient_light_route").into_owned(),
            ambient_light_mode: t!("light.ambient_light_mode").into_owned(),
            ambient_light: t!("light.ambient_light").into_owned(),
            commands: t!("light.commands").into_owned(),
            on_route: t!("light.on_route").into_owned(),
            on: t!("light.on").into_owned(),
            off_route: t!("light.off_route").into_owned(),
            off: t!("light.off").into_owned(),
            toggle_route: t!("light.toggle_route").into_owned(),
            toggle: t!("light.toggle").into_owned(),
            hazards,
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
    pub(crate) fn new(id: usize, hazards: HashMap<String, Vec<LocalizedHazard>>) -> Self {
        Self {
            id,
            info: DemoLightInfo::new(hazards),
            mode: LightMode::default(),
            has_events: false,
        }
    }

    #[inline]
    pub(crate) fn with_events(id: usize, hazards: HashMap<String, Vec<LocalizedHazard>>) -> Self {
        Self {
            id,
            info: DemoLightInfo::new(hazards),
            mode: LightMode::default(),
            has_events: true,
        }
    }

    pub(crate) fn change_state(&mut self, state: &str) {
        self.mode = match state {
            "motion" => LightMode::MotionDetection,
            "ambient" => LightMode::AmbientLight,
            // If no valid mode is passed, the default mode will be used.
            _ => LightMode::default(),
        };
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
