use std::{ops::Deref, path::Path};

use firecore_world_builder::{
    builder::structs::BuilderLocation,
    world::{
        character::npc::group::NpcGroupId,
        map::{object::ObjectId, TransitionId},
        positions::{Direction, Location},
    },
};
use hashbrown::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NameMappings {
    pub map: MapMappings,
    pub palettes: PaletteMappings,
    pub music: HashMap<String, tinystr::TinyStr16>,
    pub npcs: NpcMappings,
    pub objects: ObjectMappings,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MapMappings {
    pub id: IdMappings,
    pub name: HashMap<String, String>,
    pub transition: HashMap<String, TransitionId>,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaletteMappings {
    pub primary: HashMap<String, u8>,
    pub secondary: HashMap<String, u8>,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NpcMappings {
    pub groups: HashMap<String, NpcGroupId>,
    pub movement: HashMap<String, (bool, HashSet<Direction>)>,
}

#[derive(Default, Deserialize, Serialize)]
pub struct ObjectMappings {
    pub objects: HashMap<String, ObjectId>,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(transparent, deny_unknown_fields)]
pub struct IdMappingsFrom {
    pub inner: HashMap<String, BuilderLocation>,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(from = "IdMappingsFrom")]
pub struct IdMappings {
    inner: HashMap<String, Location>,
}

impl Deref for IdMappings {
    type Target = HashMap<String, Location>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<IdMappingsFrom> for IdMappings {
    fn from(mappings: IdMappingsFrom) -> Self {
        Self {
            inner: mappings
                .inner
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        }
    }
}

impl NameMappings {
    pub fn load() -> Self {
        let path = Path::new("./mappings.ron");

        match std::fs::read_to_string(path) {
            Ok(data) => match ron::from_str(&data) {
                Ok(mappings) => mappings,
                Err(err) => panic!("Cannot deserialize mappings with error {}", err),
            },
            Err(err) => {
                panic!("Could not load mappings file with error {}", err)
            }
        }
    }
}
