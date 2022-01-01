use dashmap::DashMap;
use firecore_world_builder::{builder::structs::BuilderLocation, world::{character::npc::NpcId, positions::Location, map::WorldMap}};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Edits {
    pub maps: HashMap<BuilderLocation, MapEdits>,
}

#[derive(Deserialize, Serialize)]
pub struct MapEdits {
    pub npcs: Vec<NpcEdits>,
}

#[derive(Deserialize, Serialize)]
pub enum NpcEdits {
    Remove(NpcId),
}

impl Edits {
    pub fn load() -> Self {
        let path = std::path::Path::new("./edits.ron");

        match std::fs::read_to_string(path) {
            Ok(data) => match ron::from_str(&data) {
                Ok(edits) => edits,
                Err(err) => panic!("Cannot deserialize edits with error {}", err),
            },
            Err(err) => {
                panic!("Could not load edits file with error {}", err)
            }
        }
    }

    pub fn process(self, maps: &DashMap<Location, WorldMap>) {
        let edits = self.maps.into_iter().map(|(k, v)| (k.into(), v)).collect::<HashMap<Location, MapEdits>>();
        for mut map in maps.iter_mut() {
            if let Some(edit) = edits.get(map.key()) {
                for npc in &edit.npcs {
                    match npc {
                        NpcEdits::Remove(id) => {
                            map.npcs.remove(id);
                        }
                    }
                }
            }
        }
    }

}
