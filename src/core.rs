pub(super) mod city;
pub(super) mod cli;
pub(super) mod control_action;
pub(super) mod developer;
pub(super) mod errors;
pub(super) mod family;
pub(super) mod game_paths;
pub(super) mod game_state;
pub(super) mod game_world;
pub(super) mod ground;
pub(super) mod orbit_camera;
pub(super) mod settings;

use bevy::{app::PluginGroupBuilder, prelude::*};

use city::CityPlugin;
use control_action::ControlActionsPlugin;
use developer::DeveloperPlugin;
use family::FamilyPlugin;
use game_paths::GamePathsPlugin;
use game_state::GameStatePlugin;
use game_world::GameWorldPlugin;
use ground::GroundPlugin;
use orbit_camera::OrbitCameraPlugin;
use settings::SettingsPlugin;

pub(super) struct CorePlugins;

impl PluginGroup for CorePlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group
            .add(GameStatePlugin)
            .add(CityPlugin)
            .add(GroundPlugin)
            .add(ControlActionsPlugin)
            .add(DeveloperPlugin)
            .add(FamilyPlugin)
            .add(GamePathsPlugin)
            .add(GameWorldPlugin)
            .add(OrbitCameraPlugin)
            .add(SettingsPlugin);
    }
}

#[cfg(test)]
mod tests {
    use bevy::{
        asset::AssetPlugin,
        core::CorePlugin,
        pbr::PbrPlugin,
        render::{settings::WgpuSettings, RenderPlugin},
        window::WindowPlugin,
    };
    use bevy_inspector_egui::WorldInspectorParams;
    use bevy_rapier3d::prelude::*;
    use leafwing_input_manager::plugin::InputManagerPlugin;

    use super::{cli::Cli, control_action::ControlAction, *};

    #[test]
    fn update() {
        App::new()
            .init_resource::<Cli>()
            .init_resource::<WorldInspectorParams>()
            .init_resource::<DebugRenderContext>()
            .add_plugin(InputManagerPlugin::<ControlAction>::default())
            .add_plugins(CorePlugins)
            .add_plugin(HeadlessRenderPlugin)
            .update();
    }

    // Allows to run tests for systems containing rendering related things without GPU
    pub(super) struct HeadlessRenderPlugin;

    impl Plugin for HeadlessRenderPlugin {
        fn build(&self, app: &mut App) {
            app.insert_resource(WgpuSettings {
                backends: None,
                ..Default::default()
            })
            .add_plugin(CorePlugin::default())
            .add_plugin(WindowPlugin::default())
            .add_plugin(AssetPlugin::default())
            .add_plugin(RenderPlugin::default())
            .add_plugin(PbrPlugin::default());
        }
    }
}
