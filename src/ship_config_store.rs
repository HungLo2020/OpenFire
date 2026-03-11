use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;

const DEFAULT_SHIP_CONFIG_PATH: &str = "assets/ships/default_ship.json";

pub struct ShipConfigStorePlugin;

impl Plugin for ShipConfigStorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ShipConfigStore::load_default(DEFAULT_SHIP_CONFIG_PATH));
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ShipConfig {
    pub display_name: String,
    pub center_of_mass_local: [f32; 3],
    pub max_speed: f32,
    pub acceleration_forward: f32,
    pub acceleration_backward: f32,
    pub acceleration_right: f32,
    pub acceleration_left: f32,
    pub acceleration_up: f32,
    pub acceleration_down: f32,
    pub roll_speed: f32,
    pub mouse_sensitivity: f32,
    pub initial_pitch: f32,
}

impl Default for ShipConfig {
    fn default() -> Self {
        Self {
            display_name: "Starter".to_string(),
            center_of_mass_local: [0.0, 0.0, 0.0],
            max_speed: 10.0,
            acceleration_forward: 24.0,
            acceleration_backward: 24.0,
            acceleration_right: 24.0,
            acceleration_left: 24.0,
            acceleration_up: 24.0,
            acceleration_down: 24.0,
            roll_speed: 1.8,
            mouse_sensitivity: 0.006,
            initial_pitch: -0.22,
        }
    }
}

#[derive(Resource, Clone)]
#[allow(dead_code)]
pub struct ShipConfigStore {
    path: String,
    default_ship: ShipConfig,
}

impl ShipConfigStore {
    pub fn load_default(path: &str) -> Self {
        let default_ship = fs::read_to_string(path)
            .ok()
            .and_then(|raw| serde_json::from_str::<ShipConfig>(&raw).ok())
            .unwrap_or_default();

        Self {
            path: path.to_string(),
            default_ship,
        }
    }

    pub fn get_default(&self) -> ShipConfig {
        self.default_ship.clone()
    }

    #[allow(dead_code)]
    pub fn set_default(&mut self, config: ShipConfig) {
        self.default_ship = config;
    }

    #[allow(dead_code)]
    pub fn save_default(&self) -> std::io::Result<()> {
        let serialized = serde_json::to_string_pretty(&self.default_ship)?;
        fs::write(&self.path, serialized)
    }
}
