use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::{
    actor::{ActorAnimation, Sex},
    animation_state::AnimationState,
    asset::collection::Collection,
    game_world::WorldName,
    navigation::Navigation,
};

pub(super) struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Movement>()
            .add_systems(
                Update,
                Self::init_system.run_if(resource_exists::<WorldName>()),
            )
            .add_systems(
                PostUpdate,
                Self::cleanup_system.run_if(resource_exists::<WorldName>()),
            );
    }
}

impl MovementPlugin {
    fn init_system(
        actor_animations: Res<Collection<ActorAnimation>>,
        mut actors: Query<(&Sex, &Navigation, &mut AnimationState), Added<Navigation>>,
    ) {
        for (sex, navigation, mut animation_state) in &mut actors {
            let animation = match sex {
                Sex::Male => {
                    if navigation.speed <= Movement::Walk.speed() {
                        ActorAnimation::MaleWalk
                    } else {
                        ActorAnimation::MaleRun
                    }
                }
                Sex::Female => {
                    if navigation.speed <= Movement::Walk.speed() {
                        ActorAnimation::FemaleWalk
                    } else {
                        ActorAnimation::FemaleRun
                    }
                }
            };

            animation_state.set_default(actor_animations.handle(animation));
        }
    }

    fn cleanup_system(
        actor_animations: Res<Collection<ActorAnimation>>,
        mut removed_navigations: RemovedComponents<Navigation>,
        mut actors: Query<&mut AnimationState>,
    ) {
        for entity in removed_navigations.read() {
            if let Ok(mut animation_state) = actors.get_mut(entity) {
                animation_state.set_default(actor_animations.handle(ActorAnimation::Idle));
            }
        }
    }
}

/// Triggers animation when the actor starts moving.
#[derive(Clone, Copy, Default, Deserialize, Reflect, Serialize)]
pub(super) enum Movement {
    #[default]
    Walk,
    Run,
}

impl Movement {
    pub(super) fn speed(self) -> f32 {
        match self {
            Movement::Walk => 2.0,
            Movement::Run => 4.0,
        }
    }
}
