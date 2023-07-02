use bevy::prelude::*;

use crate::ui::theme::Theme;

/// A simple stub just to being able to type text.
pub(super) struct TextEditPlugin;

impl Plugin for TextEditPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems((
            Self::input_system,
            Self::interaction_system,
            Self::activation_system,
        ))
        .add_system(Self::exclusive_system.in_base_set(CoreSet::PostUpdate));
    }
}

impl TextEditPlugin {
    fn input_system(
        mut char_events: EventReader<ReceivedCharacter>,
        keys: Res<Input<KeyCode>>,
        mut text_edits: Query<&mut Text, With<ActiveEdit>>,
    ) {
        if let Ok(mut text) = text_edits.get_single_mut() {
            for event in &mut char_events {
                text.sections[0].value.push(event.char);
            }
            if keys.pressed(KeyCode::Back) {
                text.sections[0].value.pop();
            }
        }
    }

    fn interaction_system(
        theme: Res<Theme>,
        mut text_edits: Query<
            (&Interaction, &mut BackgroundColor, Option<&ActiveEdit>),
            (
                Or<(Changed<Interaction>, Added<ActiveEdit>)>,
                With<TextEdit>,
            ),
        >,
    ) {
        for (&interaction, mut background, active_edit) in &mut text_edits {
            *background = match (interaction, active_edit.is_some()) {
                (Interaction::Clicked, _) | (Interaction::None, true) => {
                    theme.text_edit.active_color.into()
                }
                (Interaction::Hovered, true) => theme.text_edit.hovered_active_color.into(),
                (Interaction::Hovered, false) => theme.text_edit.hovered_color.into(),
                (Interaction::None, false) => theme.text_edit.normal_color.into(),
            };
        }
    }

    fn activation_system(
        mut commands: Commands,
        mut text_edits: Query<(Entity, &Interaction), (Changed<Interaction>, With<TextEdit>)>,
    ) {
        for (entity, &interaction) in &mut text_edits {
            if interaction == Interaction::Clicked {
                commands.entity(entity).insert(ActiveEdit);
            }
        }
    }

    fn exclusive_system(
        mut commands: Commands,
        text_edits: Query<Entity, Added<ActiveEdit>>,
        active_edits: Query<Entity, With<ActiveEdit>>,
    ) {
        for activated_entity in &text_edits {
            if let Some(edit_entity) = active_edits
                .iter()
                .find(|&entity| entity != activated_entity)
            {
                commands.entity(edit_entity).remove::<ActiveEdit>();
            }
        }
    }
}

#[derive(Bundle)]
pub(crate) struct TextEditBundle {
    text_edit: TextEdit,
    interaction: Interaction,

    #[bundle]
    button_bundle: TextBundle,
}

impl TextEditBundle {
    pub(crate) fn new(theme: &Theme, text: impl Into<String>) -> Self {
        Self {
            text_edit: TextEdit,
            interaction: Default::default(),
            button_bundle: TextBundle {
                style: theme.text_edit.style.clone(),
                text: Text::from_section(text, theme.text_edit.text.clone()),
                ..Default::default()
            },
        }
    }

    pub(crate) fn empty(theme: &Theme) -> Self {
        Self::new(theme, String::new())
    }
}

#[derive(Component)]
struct TextEdit;

#[derive(Component)]
pub(crate) struct ActiveEdit;