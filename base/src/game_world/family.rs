pub mod building;
pub mod editor;

use std::io::Cursor;

use bevy::{
    ecs::entity::{EntityMapper, MapEntities},
    prelude::*,
    reflect::serde::{ReflectDeserializer, ReflectSerializer},
    utils::HashMap,
};
use bevy_replicon::{
    core::ctx::{ClientSendCtx, ServerReceiveCtx},
    prelude::*,
};
use bincode::{DefaultOptions, ErrorKind, Options};
use serde::{de::DeserializeSeed, Deserialize, Serialize};
use strum::{Display, EnumIter};

use super::{
    actor::{Actor, ActorBundle, ReflectActorBundle, SelectedActor},
    navigation::NavigationBundle,
    WorldState,
};
use crate::{component_commands::ComponentCommandsExt, core::GameState};
use building::BuildingPlugin;
use editor::EditorPlugin;

pub struct FamilyPlugin;

impl Plugin for FamilyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((EditorPlugin, BuildingPlugin))
            .add_sub_state::<FamilyMode>()
            .enable_state_scoped_entities::<FamilyMode>()
            .register_type::<Family>()
            .register_type::<Budget>()
            .replicate::<Budget>()
            .replicate_group::<(Family, Name)>()
            .add_client_event_with(
                ChannelKind::Unordered,
                serialize_family_spawn,
                deserialize_family_spawn,
            )
            .add_mapped_client_event::<FamilyDelete>(ChannelKind::Unordered)
            .add_mapped_server_event::<SelectedFamilyCreated>(ChannelKind::Unordered)
            .add_systems(OnEnter(WorldState::Family), Self::select)
            .add_systems(OnExit(WorldState::Family), Self::deselect)
            .add_systems(
                PreUpdate,
                (
                    Self::update_members,
                    Self::init,
                    (Self::create, Self::delete).run_if(server_or_singleplayer),
                )
                    .after(ClientSet::Receive)
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

impl FamilyPlugin {
    fn init(
        mut commands: Commands,
        families: Query<Entity, (With<Family>, Without<StateScoped<GameState>>)>,
    ) {
        for entity in &families {
            commands
                .entity(entity)
                .insert(StateScoped(GameState::InGame));
        }
    }

    fn update_members(
        mut commands: Commands,
        actors: Query<(Entity, &Actor), Changed<Actor>>,
        mut families: Query<&mut FamilyMembers>,
    ) {
        let mut new_families = HashMap::<_, Vec<_>>::new();
        for (actor_entity, actor) in &actors {
            debug!("updating family for actor `{actor_entity}`");

            // Remove previous.
            for mut members in &mut families {
                if let Some(index) = members.iter().position(|&entity| entity == actor_entity) {
                    members.0.swap_remove(index);
                    break;
                }
            }

            if let Ok(mut family) = families.get_mut(actor.family_entity) {
                family.0.push(actor_entity);
            } else {
                new_families
                    .entry(actor.family_entity)
                    .or_default()
                    .push(actor_entity);
            }
        }

        // Apply accumulated `FamilyMembers` at once in case there was no such component otherwise
        // multiple `FamilyMembers` insertion with a single entity will overwrite each other.
        for (family_entity, members) in new_families {
            commands
                .entity(family_entity)
                .insert(FamilyMembers(members));
        }
    }

    fn create(
        mut commands: Commands,
        mut created_events: EventWriter<ToClients<SelectedFamilyCreated>>,
        mut create_events: ResMut<Events<FromClient<FamilyCreate>>>,
    ) {
        for FromClient { client_id, event } in create_events.drain() {
            info!("creating new family");
            let family_entity = commands
                .spawn(FamilyBundle::new(event.scene.name, event.scene.budget))
                .id();
            for actor in event.scene.actors {
                commands.entity(event.city_entity).with_children(|parent| {
                    parent
                        .spawn((
                            ParentSync::default(),
                            Transform::default(),
                            NavigationBundle::default(),
                            Actor { family_entity },
                            Replicated,
                        ))
                        .insert_reflect_bundle(actor.into_reflect());
                });
            }
            if event.select {
                created_events.send(ToClients {
                    mode: SendMode::Direct(client_id),
                    event: SelectedFamilyCreated(family_entity),
                });
            }
        }
    }

    fn delete(
        mut commands: Commands,
        mut delete_events: EventReader<FromClient<FamilyDelete>>,
        families: Query<&mut FamilyMembers>,
    ) {
        for family_entity in delete_events.read().map(|event| event.event.0) {
            match families.get(family_entity) {
                Ok(members) => {
                    info!("deleting family `{family_entity}`");
                    commands.entity(family_entity).despawn();
                    for &entity in &members.0 {
                        commands.entity(entity).despawn_recursive();
                    }
                }
                Err(e) => error!("received an invalid family to despawn: {e}"),
            }
        }
    }

    pub fn select(mut commands: Commands, actors: Query<&Actor, With<SelectedActor>>) {
        let actor = actors.single();
        info!("selecting `{}`", actor.family_entity);
        commands.entity(actor.family_entity).insert(SelectedFamily);
    }

    fn deselect(mut commands: Commands, families: Query<&Actor, With<SelectedActor>>) {
        if let Ok(actor) = families.get_single() {
            info!("deselecting `{}`", actor.family_entity);
            commands
                .entity(actor.family_entity)
                .remove::<SelectedFamily>();
        }
    }
}

fn serialize_family_spawn(
    ctx: &mut ClientSendCtx,
    event: &FamilyCreate,
    cursor: &mut Cursor<Vec<u8>>,
) -> bincode::Result<()> {
    DefaultOptions::new().serialize_into(&mut *cursor, &event.city_entity)?;
    DefaultOptions::new().serialize_into(&mut *cursor, &event.scene.name)?;
    DefaultOptions::new().serialize_into(&mut *cursor, &event.scene.budget)?;
    DefaultOptions::new().serialize_into(&mut *cursor, &event.scene.actors.len())?;
    for actor in &event.scene.actors {
        let serializer = ReflectSerializer::new(actor.as_reflect(), ctx.registry);
        DefaultOptions::new().serialize_into(&mut *cursor, &serializer)?;
    }
    DefaultOptions::new().serialize_into(cursor, &event.select)?;

    Ok(())
}

fn deserialize_family_spawn(
    ctx: &mut ServerReceiveCtx,
    cursor: &mut Cursor<&[u8]>,
) -> bincode::Result<FamilyCreate> {
    let city_entity = DefaultOptions::new().deserialize_from(&mut *cursor)?;
    let name = DefaultOptions::new().deserialize_from(&mut *cursor)?;
    let budget = DefaultOptions::new().deserialize_from(&mut *cursor)?;
    let actors_count = DefaultOptions::new().deserialize_from(&mut *cursor)?;
    let mut actors = Vec::with_capacity(actors_count);
    for _ in 0..actors_count {
        let mut deserializer =
            bincode::Deserializer::with_reader(&mut *cursor, DefaultOptions::new());
        let reflect = ReflectDeserializer::new(ctx.registry).deserialize(&mut deserializer)?;
        let type_info = reflect.get_represented_type_info().unwrap();
        let type_path = type_info.type_path();
        let registration = ctx
            .registry
            .get(type_info.type_id())
            .ok_or_else(|| ErrorKind::Custom(format!("{type_path} is not registered")))?;
        let reflect_actor = registration.data::<ReflectActorBundle>().ok_or_else(|| {
            ErrorKind::Custom(format!("{type_path} doesn't have reflect(ActorBundle)"))
        })?;
        let actor = reflect_actor
            .get_boxed(reflect)
            .map_err(|_| ErrorKind::Custom(format!("{type_path} is not an ActorBundle")))?;
        actors.push(actor);
    }
    let select = DefaultOptions::new().deserialize_from(cursor)?;

    Ok(FamilyCreate {
        city_entity,
        scene: FamilyScene {
            name,
            budget,
            actors,
        },
        select,
    })
}

#[derive(
    SubStates, Component, Clone, Copy, Debug, Eq, Hash, PartialEq, Display, EnumIter, Default,
)]
#[source(WorldState = WorldState::Family)]
pub enum FamilyMode {
    #[default]
    Life,
    Building,
}

impl FamilyMode {
    pub fn glyph(self) -> &'static str {
        match self {
            Self::Life => "👪",
            Self::Building => "🏠",
        }
    }
}

#[derive(Bundle)]
struct FamilyBundle {
    name: Name,
    budget: Budget,
    family: Family,
    replication: Replicated,
}

impl FamilyBundle {
    fn new(name: String, budget: Budget) -> Self {
        Self {
            name: Name::new(name),
            budget,
            family: Family,
            replication: Replicated,
        }
    }
}

#[derive(Component, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Family;

/// Indicates locally controlled family.
///
/// Inserted automatically on [`ActiveActor`] insertion.
#[derive(Component)]
pub struct SelectedFamily;

#[derive(Clone, Component, Copy, Default, Debug, Deserialize, Reflect, Serialize, Deref)]
#[reflect(Component)]
pub struct Budget(u32);

/// Contains the entities of all the actors that belong to the family.
///
/// Automatically created and updated based on [`ActorFamily`].
#[derive(Component, Default, Deref)]
pub struct FamilyMembers(Vec<Entity>);

#[derive(Event)]
pub struct FamilyCreate {
    pub city_entity: Entity,
    pub scene: FamilyScene,
    pub select: bool,
}

impl MapEntities for FamilyCreate {
    fn map_entities<T: EntityMapper>(&mut self, entity_mapper: &mut T) {
        self.city_entity = entity_mapper.map_entity(self.city_entity);
    }
}

#[derive(Default, Resource)]
pub struct FamilyScene {
    pub name: String,
    pub budget: Budget,
    pub actors: Vec<Box<dyn ActorBundle>>,
}

impl FamilyScene {
    pub fn new(name: String) -> Self {
        Self {
            name,
            budget: Default::default(),
            actors: Default::default(),
        }
    }
}

#[derive(Clone, Copy, Deserialize, Event, Serialize)]
pub struct FamilyDelete(pub Entity);

impl MapEntities for FamilyDelete {
    fn map_entities<T: EntityMapper>(&mut self, entity_mapper: &mut T) {
        self.0 = entity_mapper.map_entity(self.0);
    }
}

/// An event from server which indicates spawn confirmation for the selected family.
#[derive(Deserialize, Event, Serialize)]
pub(super) struct SelectedFamilyCreated(pub(super) Entity);

impl MapEntities for SelectedFamilyCreated {
    fn map_entities<T: EntityMapper>(&mut self, entity_mapper: &mut T) {
        self.0 = entity_mapper.map_entity(self.0);
    }
}
