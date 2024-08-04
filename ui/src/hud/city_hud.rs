mod lots_node;

use bevy::prelude::*;
use lots_node::LotsNodePlugin;
use strum::IntoEnumIterator;

use crate::hud::objects_node;
use project_harmonia_base::{
    asset::info::object_info::{ObjectCategory, ObjectInfo},
    game_world::{city::CityMode, WorldState},
};
use project_harmonia_widgets::{
    button::{ExclusiveButton, TabContent, TextButtonBundle, Toggled},
    theme::Theme,
};

pub(super) struct CityHudPlugin;

impl Plugin for CityHudPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LotsNodePlugin)
            .add_systems(OnEnter(WorldState::City), Self::setup)
            .add_systems(
                Update,
                Self::set_city_mode.run_if(in_state(WorldState::City)),
            );
    }
}

impl CityHudPlugin {
    fn setup(
        mut commands: Commands,
        mut tab_commands: Commands,
        theme: Res<Theme>,
        objects_info: Res<Assets<ObjectInfo>>,
    ) {
        debug!("showing city HUD");
        commands
            .spawn((
                StateScoped(WorldState::City),
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            ))
            .with_children(|parent| {
                let tabs_entity = parent
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            align_self: AlignSelf::FlexEnd,
                            padding: theme.padding.normal,
                            ..Default::default()
                        },
                        background_color: theme.panel_color.into(),
                        ..Default::default()
                    })
                    .id();

                for mode in CityMode::iter() {
                    let content_entity = parent
                        .spawn(NodeBundle {
                            style: Style {
                                align_self: AlignSelf::FlexEnd,
                                padding: theme.padding.normal,
                                column_gap: theme.gap.normal,
                                ..Default::default()
                            },
                            background_color: theme.panel_color.into(),
                            ..Default::default()
                        })
                        .with_children(|parent| match mode {
                            CityMode::Objects => {
                                objects_node::setup(
                                    parent,
                                    &mut tab_commands,
                                    &theme,
                                    &objects_info,
                                    ObjectCategory::CITY_CATEGORIES,
                                );
                            }
                            CityMode::Lots => lots_node::setup(parent, &theme),
                        })
                        .id();

                    tab_commands
                        .spawn((
                            mode,
                            TabContent(content_entity),
                            ExclusiveButton,
                            Toggled(mode == Default::default()),
                            TextButtonBundle::symbol(&theme, mode.glyph()),
                        ))
                        .set_parent(tabs_entity);
                }
            });
    }

    fn set_city_mode(
        mut city_mode: ResMut<NextState<CityMode>>,
        buttons: Query<(Ref<Toggled>, &CityMode), Changed<Toggled>>,
    ) {
        for (toggled, &mode) in &buttons {
            if toggled.0 && !toggled.is_added() {
                info!("changing city mode to `{mode:?}`");
                city_mode.set(mode);
            }
        }
    }
}
