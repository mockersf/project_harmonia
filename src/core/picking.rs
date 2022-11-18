use bevy::prelude::*;
use bevy_mod_outline::Outline;
use bevy_mod_raycast::RayCastSource;
use iyes_loopless::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use super::{action::Action, game_state::GameState, object::cursor_object};

pub(super) struct PickingPlugin;

impl Plugin for PickingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ObjectPicked>().add_system(
            Self::ray_system
                .chain(Self::object_picking_system)
                .chain(Self::outline_system)
                .run_if_not(cursor_object::cursor_object_exists)
                .run_in_state(GameState::City),
        );
    }
}

impl PickingPlugin {
    fn ray_system(
        ray_sources: Query<&RayCastSource<Pickable>>,
        parents: Query<(&Parent, Option<&Pickable>)>,
    ) -> Option<Entity> {
        for source in &ray_sources {
            if let Some((child_entity, _)) = source.intersect_top() {
                let entity = find_parent_object(child_entity, &parents)
                    .expect("object entity should have a parent");
                return Some(entity);
            }
        }

        None
    }

    fn object_picking_system(
        In(entity): In<Option<Entity>>,
        mut pick_events: EventWriter<ObjectPicked>,
        action_state: Res<ActionState<Action>>,
    ) -> Option<Entity> {
        if let Some(entity) = entity {
            if action_state.just_pressed(Action::Confirm) {
                pick_events.send(ObjectPicked(entity));
                None
            } else {
                Some(entity)
            }
        } else {
            None
        }
    }

    fn outline_system(
        In(entity): In<Option<Entity>>,
        mut previous_entity: Local<Option<Entity>>,
        mut outlines: Query<&mut Outline>,
        children: Query<&Children>,
    ) {
        if *previous_entity == entity {
            return;
        }

        if let Some(entity) = entity {
            set_outline_recursive(entity, true, &mut outlines, &children);
        }

        if let Some(entity) = *previous_entity {
            set_outline_recursive(entity, false, &mut outlines, &children);
        }

        *previous_entity = entity;
    }
}

/// Iterates up the hierarchy until it finds a parent with an [`Pickable`] component if exists.
fn find_parent_object(
    entity: Entity,
    parents: &Query<(&Parent, Option<&Pickable>)>,
) -> Option<Entity> {
    let (parent, object_path) = parents.get(entity).unwrap();
    if object_path.is_some() {
        return Some(entity);
    }

    find_parent_object(parent.get(), parents)
}

fn set_outline_recursive(
    entity: Entity,
    visible: bool,
    outlines: &mut Query<&mut Outline>,
    children: &Query<&Children>,
) {
    if let Ok(mut outline) = outlines.get_mut(entity) {
        outline.visible = visible;
    }

    if let Ok(entity_children) = children.get(entity) {
        for &entity in entity_children {
            set_outline_recursive(entity, visible, outlines, children);
        }
    }
}

#[derive(Component)]
pub(crate) struct Pickable;

pub(super) struct ObjectPicked(pub(super) Entity);

#[cfg(test)]
mod tests {
    use bevy::{asset::AssetPlugin, core::CorePlugin, ecs::system::SystemState};
    use bevy_mod_raycast::IntersectionData;

    use super::*;

    #[test]
    fn parent_search() {
        let mut world = World::new();
        let child_entity = world.spawn().id();
        let parent_entity = world
            .spawn()
            .insert(Pickable)
            .push_children(&[child_entity])
            .id();

        // Assign a parent, as an outline object is always expected to have a parent object.
        world.spawn().push_children(&[parent_entity]);

        let mut system_state: SystemState<Query<(&Parent, Option<&Pickable>)>> =
            SystemState::new(&mut world);

        let entity = find_parent_object(child_entity, &system_state.get(&world))
            .expect("object should have a parent");
        assert_eq!(entity, parent_entity);
    }

    #[test]
    fn recursive_outline() {
        let mut world = World::new();
        let child_entity1 = world.spawn().insert(Outline::default()).id();
        let child_entity2 = world
            .spawn()
            .insert(Outline::default())
            .push_children(&[child_entity1])
            .id();
        let root_entity = world
            .spawn()
            .insert(Outline::default())
            .push_children(&[child_entity2])
            .id();

        let mut system_state: SystemState<(Query<&mut Outline>, Query<&Children>)> =
            SystemState::new(&mut world);

        const VISIBLE: bool = false;
        let (mut outlines, children) = system_state.get_mut(&mut world);
        set_outline_recursive(root_entity, VISIBLE, &mut outlines, &children);

        assert_eq!(
            world.get::<Outline>(child_entity1).unwrap().visible,
            VISIBLE
        );
        assert_eq!(
            world.get::<Outline>(child_entity2).unwrap().visible,
            VISIBLE
        );
        assert_eq!(world.get::<Outline>(root_entity).unwrap().visible, VISIBLE);
    }

    #[test]
    fn hovering() {
        let mut app = App::new();
        app.add_loopless_state(GameState::City)
            .init_resource::<ActionState<Action>>()
            .add_plugin(CorePlugin)
            .add_plugin(AssetPlugin)
            .add_plugin(PickingPlugin);

        let outline_entity = app
            .world
            .spawn()
            .insert(Outline::default())
            .insert(Pickable)
            .id();
        app.world.spawn().push_children(&[outline_entity]);

        let mut ray_source = RayCastSource::<Pickable>::default();
        ray_source.intersections_mut().push((
            outline_entity,
            IntersectionData::new(Vec3::default(), Vec3::default(), 0.0, None),
        ));
        let ray_entity = app.world.spawn().insert(ray_source).id();

        app.update();

        assert!(app.world.get::<Outline>(outline_entity).unwrap().visible);

        let next_outline_entity = app
            .world
            .spawn()
            .insert(Outline::default())
            .insert(Pickable)
            .id();
        app.world.spawn().push_children(&[next_outline_entity]);
        let mut ray_source = app
            .world
            .get_mut::<RayCastSource<Pickable>>(ray_entity)
            .unwrap();
        ray_source.intersections_mut().clear();
        ray_source.intersections_mut().push((
            next_outline_entity,
            IntersectionData::new(Vec3::default(), Vec3::default(), 0.0, None),
        ));

        app.update();

        assert!(!app.world.get::<Outline>(outline_entity).unwrap().visible);
        assert!(
            app.world
                .get::<Outline>(next_outline_entity)
                .unwrap()
                .visible
        );
    }
}
