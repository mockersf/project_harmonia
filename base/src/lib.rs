pub mod asset;
mod combined_scene_collider;
pub mod common_conditions;
mod component_commands;
pub mod core;
pub mod game_paths;
pub mod game_world;
mod ghost;
pub mod input_events;
mod math;
pub mod message;
pub mod network;
pub mod settings;

use bevy::{app::PluginGroupBuilder, prelude::*};

use asset::AssetPlugin;
use combined_scene_collider::SceneColliderConstructorPlugin;
use core::CorePlugin;
use game_paths::GamePathsPlugin;
use game_world::GameWorldPlugin;
use math::MathPlugin;
use message::ErrorReportPlugin;
use settings::SettingsPlugin;

pub struct CorePlugins;

impl PluginGroup for CorePlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(AssetPlugin)
            .add(MathPlugin)
            .add(CorePlugin)
            .add(SceneColliderConstructorPlugin)
            .add(GameWorldPlugin)
            .add(ErrorReportPlugin)
            .add(GamePathsPlugin)
            .add(SettingsPlugin)
    }
}
