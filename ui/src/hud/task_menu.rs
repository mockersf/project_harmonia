use bevy::prelude::*;
use leafwing_input_manager::common_conditions::action_just_pressed;

use project_harmonia_base::{
    game_world::{
        actor::{
            task::{Task, TaskList, TaskRequest},
            SelectedActor,
        },
        family::FamilyMode,
        hover::Hovered,
    },
    settings::Action,
};
use project_harmonia_widgets::{
    button::TextButtonBundle, click::Click, label::LabelBundle, theme::Theme,
};

pub(super) struct TaskMenuPlugin;

impl Plugin for TaskMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            Self::request_task.run_if(in_state(FamilyMode::Life)),
        )
        .add_systems(
            PostUpdate,
            (
                Self::open,
                Self::close.run_if(action_just_pressed(Action::Cancel)),
            )
                .run_if(in_state(FamilyMode::Life)),
        );
    }
}

impl TaskMenuPlugin {
    fn open(
        mut commands: Commands,
        mut list_events: ResMut<Events<TaskList>>,
        theme: Res<Theme>,
        task_menus: Query<Entity, With<TaskMenu>>,
        hovered: Query<&Name, With<Hovered>>,
        windows: Query<&Window>,
        roots: Query<Entity, (With<Node>, Without<Parent>)>,
    ) {
        let tasks = list_events.drain().map(|event| event.0).collect::<Vec<_>>();
        if tasks.is_empty() {
            return;
        }

        if let Ok(entity) = task_menus.get_single() {
            info!("reopening task menu");
            commands.entity(entity).despawn_recursive();
        } else {
            info!("displaying task menu");
        }

        let cursor_pos = windows.single().cursor_position().unwrap_or_default();
        commands.entity(roots.single()).with_children(|parent| {
            parent
                .spawn_empty()
                .with_children(|parent| {
                    parent.spawn(LabelBundle::normal(&theme, hovered.single()));

                    for (index, task) in tasks.iter().enumerate() {
                        parent.spawn((
                            TaskMenuIndex(index),
                            TextButtonBundle::normal(&theme, task.name()),
                        ));
                    }
                })
                .insert((
                    TaskMenu(tasks),
                    NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            left: Val::Px(cursor_pos.x),
                            top: Val::Px(cursor_pos.y),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            padding: theme.padding.normal,
                            row_gap: theme.gap.normal,
                            ..Default::default()
                        },
                        background_color: theme.panel_color.into(),
                        ..Default::default()
                    },
                ));
        });
    }

    fn request_task(
        mut commands: Commands,
        mut send_requests: EventWriter<TaskRequest>,
        mut click_events: EventReader<Click>,
        buttons: Query<&TaskMenuIndex>,
        mut task_menus: Query<(Entity, &mut TaskMenu)>,
        active_actors: Query<Entity, With<SelectedActor>>,
    ) {
        for task_index in buttons.iter_many(click_events.read().map(|event| event.0)) {
            let (menu_entity, mut task_menu) = task_menus.single_mut();
            let task = task_menu.swap_remove(task_index.0);

            info!("selecting task '{}'", task.name());
            send_requests.send(TaskRequest {
                entity: active_actors.single(),
                task,
            });

            commands.entity(menu_entity).despawn_recursive();
        }
    }

    fn close(mut commands: Commands, task_menus: Query<Entity, With<TaskMenu>>) {
        if let Ok(entity) = task_menus.get_single() {
            info!("closing task menu");
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[derive(Component, Deref, DerefMut)]
pub(crate) struct TaskMenu(Vec<Box<dyn Task>>);

#[derive(Component)]
struct TaskMenuIndex(usize);
