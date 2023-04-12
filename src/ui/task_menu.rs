use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{egui::Pos2, EguiContexts};
use bevy_inspector_egui::egui::{Align, Layout};
use bevy_trait_query::One;

use crate::core::{
    family::FamilyMode,
    game_state::GameState,
    task::{Task, TaskList, TaskRequest},
};

pub(super) struct TaskMenuPlugin;

impl Plugin for TaskMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            Self::menu_system
                .in_set(OnUpdate(GameState::Family))
                .in_set(OnUpdate(FamilyMode::Life)),
        );
    }
}

impl TaskMenuPlugin {
    fn menu_system(
        mut position: Local<Pos2>,
        mut commands: Commands,
        mut egui: EguiContexts,
        mut task_events: EventWriter<TaskRequest>,
        windows: Query<&Window, With<PrimaryWindow>>,
        task_lists: Query<(Entity, &Name, Ref<TaskList>, Option<&Children>)>,
        tasks: Query<(Entity, One<&dyn Task>)>,
    ) {
        let Ok((entity, name, task_list, children)) = task_lists.get_single() else {
            return;
        };

        if task_list.is_added() {
            // Recalculate window position.
            let primary_window = windows.single();
            let cursor_position = primary_window.cursor_position().unwrap_or_default();
            position.x = cursor_position.x;
            position.y = primary_window.height() - cursor_position.y;
        }

        let mut task_activated = false;
        let mut open = true;
        bevy_egui::egui::Window::new(name.as_str())
            .resizable(false)
            .collapsible(false)
            .fixed_pos(*position)
            .default_width(130.0)
            .open(&mut open)
            .show(egui.ctx_mut(), |ui| {
                ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
                    for (_, task) in
                        tasks.iter_many(children.iter().flat_map(|children| children.iter()))
                    {
                        if ui.button(task.name()).clicked() {
                            task_events.send(task.to_request());
                            task_activated = true;
                        }
                    }
                });
            });

        if !open || task_activated {
            commands.entity(entity).remove::<TaskList>();
            for (task_entity, _) in
                tasks.iter_many(children.iter().flat_map(|children| children.iter()))
            {
                commands.entity(task_entity).despawn();
            }
        }
    }
}
