use std::{
    any,
    fmt::{self, Formatter},
    path::PathBuf,
};

use bevy::{
    prelude::*,
    reflect::{serde::TypedReflectDeserializer, TypeRegistry},
    scene::ron::{self, error::SpannedResult},
};
use serde::{
    de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use strum::{Display, IntoStaticStr, VariantNames};

use super::{GeneralMetadata, Metadata};

#[derive(TypePath, Asset)]
pub struct ObjectMetadata {
    pub general: GeneralMetadata,
    pub category: ObjectCategory,
    pub preview_translation: Vec3,
    pub components: Vec<Box<dyn Reflect>>,
    pub place_components: Vec<Box<dyn Reflect>>,
    pub spawn_components: Vec<Box<dyn Reflect>>,
}

impl Metadata for ObjectMetadata {
    const DIR: &'static str = "objects";

    fn from_str(data: &str, options: ron::Options, registry: &TypeRegistry) -> SpannedResult<Self> {
        options.from_str_seed(data, ObjectMetadataDeserializer { registry })
    }

    fn iter_paths_mut(&mut self) -> impl Iterator<Item = &mut PathBuf> {
        [&mut self.general.asset].into_iter()
    }
}

/// Fields of [`ObjectMetadata`] for manual deserialization.
#[derive(Deserialize, VariantNames, IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
#[serde(field_identifier, rename_all = "snake_case")]
enum ObjectMetadataField {
    General,
    Category,
    PreviewTranslation,
    Components,
    PlaceComponents,
    SpawnComponents,
}

#[derive(Clone, Component, Copy, Deserialize, Display, PartialEq)]
pub enum ObjectCategory {
    Rocks,
    Foliage,
    OutdoorFurniture,
    OutdoorActivities,
    Street,
    Electronics,
    Furniture,
    Windows,
    Doors,
}

impl ObjectCategory {
    pub const CITY_CATEGORIES: &'static [ObjectCategory] = &[
        ObjectCategory::Rocks,
        ObjectCategory::Foliage,
        ObjectCategory::OutdoorFurniture,
        ObjectCategory::OutdoorActivities,
        ObjectCategory::Street,
    ];

    pub const FAMILY_CATEGORIES: &'static [ObjectCategory] = &[
        ObjectCategory::Rocks,
        ObjectCategory::Foliage,
        ObjectCategory::OutdoorFurniture,
        ObjectCategory::Electronics,
        ObjectCategory::Furniture,
        ObjectCategory::Windows,
        ObjectCategory::Doors,
    ];

    pub fn glyph(self) -> &'static str {
        match self {
            ObjectCategory::Rocks => "🗻",
            ObjectCategory::Foliage => "🍀",
            ObjectCategory::OutdoorFurniture => "🏡",
            ObjectCategory::OutdoorActivities => "🔤",
            ObjectCategory::Street => "🚃",
            ObjectCategory::Electronics => "📺",
            ObjectCategory::Furniture => "💺",
            ObjectCategory::Windows => "🔲",
            ObjectCategory::Doors => "🚪",
        }
    }
}

pub(super) struct ObjectMetadataDeserializer<'a> {
    registry: &'a TypeRegistry,
}

impl<'de> DeserializeSeed<'de> for ObjectMetadataDeserializer<'_> {
    type Value = ObjectMetadata;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_struct(
            any::type_name::<Self::Value>(),
            ObjectMetadataField::VARIANTS,
            self,
        )
    }
}

impl<'de> Visitor<'de> for ObjectMetadataDeserializer<'_> {
    type Value = ObjectMetadata;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str(any::type_name::<Self::Value>())
    }

    fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<Self::Value, V::Error> {
        let mut general = None;
        let mut category = None;
        let mut preview_translation = None;
        let mut components = None;
        let mut place_components = None;
        let mut spawn_components = None;
        while let Some(key) = map.next_key()? {
            match key {
                ObjectMetadataField::General => {
                    if general.is_some() {
                        return Err(de::Error::duplicate_field(
                            ObjectMetadataField::General.into(),
                        ));
                    }
                    general = Some(map.next_value()?);
                }
                ObjectMetadataField::Category => {
                    if category.is_some() {
                        return Err(de::Error::duplicate_field(
                            ObjectMetadataField::Category.into(),
                        ));
                    }
                    category = Some(map.next_value()?);
                }
                ObjectMetadataField::PreviewTranslation => {
                    if preview_translation.is_some() {
                        return Err(de::Error::duplicate_field(
                            ObjectMetadataField::PreviewTranslation.into(),
                        ));
                    }
                    preview_translation = Some(map.next_value()?);
                }
                ObjectMetadataField::Components => {
                    if components.is_some() {
                        return Err(de::Error::duplicate_field(
                            ObjectMetadataField::Components.into(),
                        ));
                    }
                    components =
                        Some(map.next_value_seed(ComponentsDeserializer::new(self.registry))?);
                }
                ObjectMetadataField::PlaceComponents => {
                    if place_components.is_some() {
                        return Err(de::Error::duplicate_field(
                            ObjectMetadataField::PlaceComponents.into(),
                        ));
                    }
                    place_components =
                        Some(map.next_value_seed(ComponentsDeserializer::new(self.registry))?);
                }
                ObjectMetadataField::SpawnComponents => {
                    if spawn_components.is_some() {
                        return Err(de::Error::duplicate_field(
                            ObjectMetadataField::SpawnComponents.into(),
                        ));
                    }
                    spawn_components =
                        Some(map.next_value_seed(ComponentsDeserializer::new(self.registry))?);
                }
            }
        }

        let general =
            general.ok_or_else(|| de::Error::missing_field(ObjectMetadataField::General.into()))?;
        let category = category
            .ok_or_else(|| de::Error::missing_field(ObjectMetadataField::Category.into()))?;
        let preview_translation = preview_translation.ok_or_else(|| {
            de::Error::missing_field(ObjectMetadataField::PreviewTranslation.into())
        })?;
        let components = components.unwrap_or_default();
        let place_components = place_components.unwrap_or_default();
        let spawn_components = spawn_components.unwrap_or_default();

        Ok(ObjectMetadata {
            general,
            category,
            preview_translation,
            components,
            place_components,
            spawn_components,
        })
    }
}

struct ComponentsDeserializer<'a> {
    registry: &'a TypeRegistry,
}

impl<'a> ComponentsDeserializer<'a> {
    fn new(registry: &'a TypeRegistry) -> Self {
        Self { registry }
    }
}

impl<'de> DeserializeSeed<'de> for ComponentsDeserializer<'_> {
    type Value = Vec<Box<dyn Reflect>>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_seq(self)
    }
}

impl<'de> Visitor<'de> for ComponentsDeserializer<'_> {
    type Value = Vec<Box<dyn Reflect>>;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str(any::type_name::<Self::Value>())
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut components = Vec::with_capacity(seq.size_hint().unwrap_or_default());
        while let Some(component) =
            seq.next_element_seed(ShortReflectDeserializer::new(self.registry))?
        {
            components.push(component);
        }

        Ok(components)
    }
}

/// Like [`UntypedReflectDeserializer`], but searches for registration by short name.
pub(super) struct ShortReflectDeserializer<'a> {
    registry: &'a TypeRegistry,
}

impl<'a> ShortReflectDeserializer<'a> {
    fn new(registry: &'a TypeRegistry) -> Self {
        Self { registry }
    }
}

impl<'de> DeserializeSeed<'de> for ShortReflectDeserializer<'_> {
    type Value = Box<dyn Reflect>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_map(self)
    }
}

impl<'de> Visitor<'de> for ShortReflectDeserializer<'_> {
    type Value = Box<dyn Reflect>;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str(any::type_name::<Self::Value>())
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let type_path = map
            .next_key::<String>()?
            .ok_or_else(|| de::Error::invalid_length(0, &"at least one entry"))?;
        let registration = self
            .registry
            .get_with_short_type_path(&type_path)
            .ok_or_else(|| de::Error::custom(format!("{type_path} is not registered")))?;
        let reflect =
            map.next_value_seed(TypedReflectDeserializer::new(registration, self.registry))?;

        Ok(reflect)
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        let registration = self
            .registry
            .get_with_short_type_path(v)
            .ok_or_else(|| de::Error::custom(format!("{v} is not registered")))?;
        let reflect_default = registration
            .data::<ReflectDefault>()
            .ok_or_else(|| de::Error::custom(format!("{v} doesn't have reflect(Default)")))?;

        Ok(reflect_default.default())
    }
}
