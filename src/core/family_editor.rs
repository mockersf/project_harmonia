use bevy::prelude::*;
use iyes_loopless::prelude::*;

use super::{family::FamilyBundle, game_state::GameState, orbit_camera::OrbitCameraBundle};

pub(super) struct FamilyEditorPlugin;

impl Plugin for FamilyEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::FamilyEditor, Self::spawn_system)
            .add_exit_system(GameState::FamilyEditor, Self::cleanup_system)
            .add_system(Self::visibility_enable_system.run_in_state(GameState::FamilyEditor))
            .add_system_to_stage(
                CoreStage::PostUpdate,
                Self::visibility_disable_system.run_in_state(GameState::FamilyEditor),
            );
    }
}

impl FamilyEditorPlugin {
    fn spawn_system(mut commands: Commands) {
        commands
            .spawn_bundle(FamilyEditorBundle::default())
            .with_children(|parent| {
                parent.spawn_bundle(PointLightBundle {
                    point_light: PointLight {
                        intensity: 1500.0,
                        shadows_enabled: true,
                        shadow_depth_bias: 0.25,
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(4.0, 8.0, 4.0),
                    ..Default::default()
                });
                parent.spawn_bundle(OrbitCameraBundle::default());
                parent
                    .spawn_bundle(FamilyBundle::default())
                    .insert(EditableFamily);
            });
    }

    fn visibility_enable_system(
        mut new_editable_dolls: Query<&mut Visibility, Added<EditableDoll>>,
    ) {
        for mut visibility in &mut new_editable_dolls {
            visibility.is_visible = true;
        }
    }

    fn visibility_disable_system(
        removed_editable_dolls: RemovedComponents<EditableDoll>,
        mut visibility: Query<&mut Visibility>,
    ) {
        for entity in removed_editable_dolls.iter() {
            let mut visibility = visibility.get_mut(entity).unwrap();
            visibility.is_visible = false;
        }
    }

    fn cleanup_system(mut commands: Commands, family_editors: Query<Entity, With<FamilyEditor>>) {
        commands.entity(family_editors.single()).despawn_recursive();
    }
}

#[derive(Bundle)]
struct FamilyEditorBundle {
    name: Name,
    family_editor: FamilyEditor,

    #[bundle]
    spatial_bundle: SpatialBundle,
}

impl Default for FamilyEditorBundle {
    fn default() -> Self {
        Self {
            name: Name::new("Family editor"),
            family_editor: FamilyEditor,
            spatial_bundle: Default::default(),
        }
    }
}

/// A root family editor component.
#[derive(Component, Default)]
pub(crate) struct FamilyEditor;

/// Currently editing family.
#[derive(Component)]
pub(crate) struct EditableFamily;

/// Currently editing doll.
#[derive(Component)]
pub(crate) struct EditableDoll;