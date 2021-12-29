use std::{collections::HashMap, ops::Deref, path::Path};

use firecore_world_builder::{
    builder::location::MapLocation,
    world::{
        character::npc::{group::NpcGroupId, NpcMovement},
        positions::{Direction, Location},
    },
};
use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NameMappings {
    pub map: MapMappings,
    pub palettes: PaletteMappings,
    pub music: HashMap<String, tinystr::TinyStr16>,
    pub npcs: NpcMappings,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MapMappings {
    pub id: IdMappings,
    pub name: HashMap<String, String>,
}

#[derive(Default, Deserialize, Serialize)]
pub struct NpcMappings {
    pub groups: HashMap<String, NpcGroupId>,
    pub movement: HashMap<String, (NpcMovement, Direction)>,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(transparent, deny_unknown_fields)]
pub struct IdMappingsFrom {
    pub inner: HashMap<String, MapLocation>,
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

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaletteMappings {
    pub primary: HashMap<String, u8>,
    pub secondary: HashMap<String, u8>,
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
