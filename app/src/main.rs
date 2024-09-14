mod cli;

use avian3d::{prelude::*, sync::SyncConfig};
use bevy::{
    core_pipeline::experimental::taa::TemporalAntiAliasPlugin,
    pbr::wireframe::WireframePlugin,
    prelude::*,
    render::{
        settings::{RenderCreation, WgpuFeatures, WgpuSettings},
        RenderPlugin,
    },
};
use bevy_atmosphere::prelude::*;
#[cfg(feature = "inspector")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_outline::OutlinePlugin;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::RepliconRenetPlugins;
use bevy_simple_text_input::TextInputPlugin;
use leafwing_input_manager::prelude::*;
use project_harmonia_base::{game_world::navigation::Obstacle, settings::Action, CorePlugins};
use project_harmonia_ui::UiPlugins;
use project_harmonia_widgets::WidgetsPlugin;
use vleue_navigator::prelude::*;

use cli::{Cli, CliPlugin};

fn main() {
    let mut app = App::new();
    app.init_resource::<Cli>()
        .insert_resource(SyncConfig {
            position_to_transform: false,
            ..Default::default()
        })
        .add_plugins((
            DefaultPlugins.set(RenderPlugin {
                render_creation: RenderCreation::Automatic(WgpuSettings {
                    features: WgpuFeatures::POLYGON_MODE_LINE,
                    ..Default::default()
                }),
                synchronous_pipeline_compilation: true,
            }),
            TemporalAntiAliasPlugin,
            RepliconPlugins,
            RepliconRenetPlugins,
            WireframePlugin,
            AtmospherePlugin,
            InputManagerPlugin::<Action>::default(),
            VleueNavigatorPlugin,
            NavmeshUpdaterPlugin::<Collider, Obstacle>::default(),
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
            TextInputPlugin,
            OutlinePlugin,
        ))
        .add_plugins((CliPlugin, CorePlugins, WidgetsPlugin, UiPlugins));

    #[cfg(feature = "inspector")]
    app.add_plugins(WorldInspectorPlugin::default());

    app.run();
}
