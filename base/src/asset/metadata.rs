pub mod object_metadata;

use std::{
    env, fs,
    marker::PhantomData,
    path::{Path, PathBuf},
};

use anyhow::Result;
use bevy::{
    app::PluginGroupBuilder,
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    reflect::{TypeRegistry, TypeRegistryArc},
    scene::ron::{self, error::SpannedResult},
};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use object_metadata::ObjectMetadata;

pub(super) struct MetadataPlugins;

impl PluginGroup for MetadataPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(MetadataPlugin::<ObjectMetadata>::default())
    }
}

struct MetadataPlugin<T>(PhantomData<T>);

impl<T> Default for MetadataPlugin<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T: Asset + Metadata> Plugin for MetadataPlugin<T> {
    fn build(&self, app: &mut App) {
        app.init_asset::<T>()
            .init_asset_loader::<MetadataLoader<T>>()
            .init_resource::<MetadataHandles<T>>();
    }
}

pub struct MetadataLoader<T> {
    registry: TypeRegistryArc,
    marker: PhantomData<T>,
}

impl<T> FromWorld for MetadataLoader<T> {
    fn from_world(world: &mut World) -> Self {
        Self {
            registry: world.resource::<AppTypeRegistry>().0.clone(),
            marker: PhantomData,
        }
    }
}

const METADATA_EXTENSION: &str = "info.ron";

impl<T: Asset + Metadata> AssetLoader for MetadataLoader<T> {
    type Asset = T;
    type Settings = ();
    type Error = anyhow::Error;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut data = String::new();
        reader.read_to_string(&mut data).await?;

        let mut metadata = T::from_str(&data, ron::Options::default(), &self.registry.read())?;
        if let Some(dir) = load_context.path().parent() {
            for path in metadata.iter_paths_mut() {
                *path = dir.join(&*path);
            }
        }

        Ok(metadata)
    }

    fn extensions(&self) -> &[&str] {
        &[METADATA_EXTENSION]
    }
}

/// Preloads and stores metadata handles.
#[derive(Resource)]
#[allow(dead_code)]
struct MetadataHandles<T: Asset>(Vec<Handle<T>>);

impl<T: Asset + Metadata> FromWorld for MetadataHandles<T> {
    fn from_world(world: &mut World) -> Self {
        let assets_dir =
            Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap_or_default()).join("assets");

        let mut handles = Vec::new();
        let asset_server = world.resource::<AssetServer>();
        for mut dir in fs::read_dir(&assets_dir)
            .expect("unable to read assets")
            .flat_map(|entry| entry.ok())
            .map(|entry| entry.path())
        {
            dir.push(T::DIR);

            for entry in WalkDir::new(&dir)
                .into_iter()
                .filter_map(|entry| entry.ok())
            {
                // Use `ends_with` because extension consists of 2 dots.
                if entry
                    .path()
                    .to_str()
                    .is_some_and(|path| path.ends_with(METADATA_EXTENSION))
                {
                    let path = entry
                        .path()
                        .strip_prefix(&assets_dir)
                        .unwrap_or_else(|e| panic!("entries should start with {dir:?}: {e}"));

                    debug!("loading metadata for {path:?}");
                    handles.push(asset_server.load(path.to_path_buf()));
                }
            }
        }

        Self(handles)
    }
}

trait Metadata: Sized {
    /// Directory from which files should be preloaded.
    const DIR: &'static str;

    fn from_str(data: &str, options: ron::Options, registry: &TypeRegistry) -> SpannedResult<Self>;

    /// Returns iterator over mutable references of all paths.
    ///
    /// Needed to convert from paths relative to the file into absolute paths.
    fn iter_paths_mut(&mut self) -> impl Iterator<Item = &mut PathBuf>;
}

#[derive(Serialize, Deserialize)]
pub struct GeneralMetadata {
    pub name: String,
    pub asset: PathBuf,
    pub author: String,
    pub license: String,
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use anyhow::{Context, Result};
    use bevy::scene::ron;
    use walkdir::WalkDir;

    use super::*;
    use crate::{
        asset::metadata::METADATA_EXTENSION,
        game_world::object::{
            door::Door,
            placing_object::{side_snap::SideSnap, wall_snap::WallSnap},
            wall_mount::WallMount,
        },
    };

    #[test]
    fn deserialization() -> Result<()> {
        let mut registry = TypeRegistry::new();
        registry.register::<Vec2>();
        registry.register::<Vec<Vec2>>();
        registry.register::<WallMount>();
        registry.register::<WallSnap>();
        registry.register::<SideSnap>();
        registry.register::<Door>();

        deserialize::<ObjectMetadata>(&registry)?;

        Ok(())
    }

    fn deserialize<A: Metadata>(registry: &TypeRegistry) -> Result<()> {
        let assets_dir = Path::new("../app/assets/base").join(A::DIR);
        for entry in WalkDir::new(assets_dir)
            .into_iter()
            .filter_map(|entry| entry.ok())
        {
            // Use `ends_with` because extension consists of 2 dots.
            if entry
                .path()
                .to_str()
                .is_some_and(|path| path.ends_with(METADATA_EXTENSION))
            {
                let data = fs::read_to_string(entry.path())?;
                A::from_str(&data, ron::Options::default(), &registry)
                    .with_context(|| format!("unable to parse {:?}", entry.path()))?;
            }
        }

        Ok(())
    }
}
