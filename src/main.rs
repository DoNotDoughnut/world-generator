use dashmap::DashMap;
use hashbrown::HashMap;

use firecore_world_builder::{
    bin::BinaryMap,
    world::{
        character::{
            npc::{Npc, NpcInteract, Npcs},
            Character,
        },
        map::{
            chunk::{Connection, WorldChunk},
            warp::{WarpDestination, WarpEntry, WarpId, WarpTransition},
            wild::{WildEntry, WildType},
            PaletteId, WorldMap,
        },
        pokedex::{item::Item, moves::Move, pokemon::Pokemon, BasicDex},
        positions::{BoundingBox, Coordinate, Destination, Direction, Location, Position},
    },
};
use map::{
    object::JsonObjectEvents, warp::JsonWarpEvent, wild::JsonWildEncounters, JsonConnection,
    JsonMap,
};
use mapping::NameMappings;
use rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};
use script_parser::Script;
use serde_json::Value;
use tinystr::{tinystr16, TinyStr16};

const PATH: &str = "http://raw.githubusercontent.com/pret/pokefirered/master";

const PARSED: &str = "parsed.bin";

mod map;
mod mapping;
mod serializable;

type Scripts = DashMap<String, Script>;
type Messages = DashMap<String, Vec<Vec<String>>>;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ParsedData {
    maps: DashMap<String, JsonMap>,
    wild: JsonWildEncounters,
    dex: BasicDex<Pokemon>,
    scripts: Scripts,
    messages: Messages,
}

fn main() {
    let mappings = mapping::NameMappings::load();

    let data = match std::fs::read(PARSED)
        .ok()
        .map(|bytes| bincode::deserialize(&bytes).ok())
        .flatten()
    {
        Some(data) => data,
        None => {
            eprintln!("Parsed map file cannot be read!");
            eprintln!("Generating new parsed map file...");

            println!("Loading dex...");

            let dex = std::fs::read("dex.bin").unwrap();

            let (dex, ..) =
                bincode::deserialize::<(BasicDex<Pokemon>, BasicDex<Move>, BasicDex<Item>)>(&dex)
                    .unwrap();

            println!("Getting layouts...");

            let layouts = attohttpc::get(format!("{}/data/layouts/layouts.json", PATH))
                .send()
                .unwrap()
                .json::<map::JsonMapLayouts>()
                .unwrap();

            println!("Getting map groups...");

            let maps = attohttpc::get(format!("{}/data/maps/map_groups.json", PATH))
                .send()
                .unwrap()
                .bytes()
                .unwrap();

            println!("Getting wild encounters...");

            let wild = attohttpc::get(format!("{}/src/data/wild_encounters.json", PATH))
                .send()
                .unwrap()
                .json::<JsonWildEncounters>()
                .unwrap();

            println!("Parsing map groups...");

            let maps = serde_json::from_slice::<Value>(&maps).unwrap();

            let mut names = Vec::new();

            for group_name in maps.get("group_order").unwrap().as_array().unwrap() {
                for name in maps
                    .get(group_name.as_str().unwrap())
                    .unwrap()
                    .as_array()
                    .unwrap()
                {
                    names.push(name.as_str().unwrap());
                }
            }

            println!("Found {} map names", names.len());

            let maps = DashMap::new();
            let scripts = DashMap::new();
            let messages = DashMap::new();

            let layouts = layouts
                .layouts
                .into_iter()
                .flat_map(|l| l.inner.left())
                .map(|l| (l.id.clone(), l))
                .collect::<DashMap<_, _>>();

            names.into_par_iter().for_each(|map| {
                let path = format!("{}/data/maps/{}/map.json", PATH, map);
                let scripts_path = format!("{}/data/maps/{}/scripts.inc", PATH, map);
                let text_path = format!("{}/data/maps/{}/text.inc", PATH, map);

                let data = attohttpc::get(path)
                    .send()
                    .unwrap_or_else(|err| panic!("Could not get {} with error {}", map, err))
                    .json::<map::JsonMapData>()
                    .unwrap_or_else(|err| panic!("Could not get {} with error {}", map, err));

                if let Some(scripts_data) = attohttpc::get(scripts_path)
                    .send()
                    .ok()
                    .map(|r| r.text().ok())
                    .flatten()
                {
                    if let Ok(scripts_data) = script_parser::parse(&scripts_data) {
                        for script in scripts_data {
                            scripts.insert(script.name.clone(), script);
                        }
                    }
                }

                if let Some(message_data) = attohttpc::get(text_path)
                    .send()
                    .ok()
                    .map(|r| r.text().ok())
                    .flatten()
                {
                    if let Ok(message_data) = script_parser::parse_message_script(&message_data) {
                        for message in message_data {
                            messages.insert(message.name, message.text);
                        }
                    }
                }

                let layout = layouts
                    .get(&data.layout)
                    .unwrap_or_else(|| panic!("Could not get map layout {}", data.layout))
                    .clone();

                println!("Parsed map {}", data.name);

                if let Some(removed) = maps.insert(data.id.clone(), JsonMap { data, layout }) {
                    panic!("Map {} was removed!", removed.data.name);
                }
            });

            let data = ParsedData {
                maps,
                wild,
                dex,
                scripts,
                messages,
            };

            println!("Done parsing maps!");

            std::fs::write("parsed.bin", bincode::serialize(&data).unwrap()).unwrap();

            data
        }
    };

    let ParsedData {
        maps,
        wild,
        dex,
        scripts,
        messages,
    } = data;

    println!("Converting wild encounters...");

    eprintln!("TODO: fix fishing encounters");

    let encounters = DashMap::new();

    wild.wild_encounter_groups
        .into_par_iter()
        .flat_map(|g| g.encounters.into_par_iter())
        .filter(|e| e.base_label[(e.base_label.len() - 7)..].eq_ignore_ascii_case("FireRed"))
        .for_each(|e| {
            let mut entries = HashMap::new();
            if let Some(e) = e.land_mons {
                entries.insert(WildType::Land, e.into(&dex));
            }
            if let Some(e) = e.water_mons {
                entries.insert(WildType::Water, e.into(&dex));
            }
            if let Some(e) = e.rock_smash_mons {
                entries.insert(WildType::Rock, e.into(&dex));
            }
            if let Some(e) = e.fishing_mons {
                entries.insert(WildType::Fishing(0), e.into(&dex));
            }
            if entries.is_empty() {
                encounters.insert(e.map, None);
            } else {
                encounters.insert(e.map, Some(entries));
            }
        });

    println!("Created {} wild encounters", encounters.len());

    let new_maps = DashMap::<Location, WorldMap>::new();

    println!("Converting maps...");

    maps.iter().par_bridge().for_each(|map| {
        let map = map.value();
        println!("Converting {}", map.data.name);
        if let Some(map) = into_world_map(&mappings, &scripts, &messages, &maps, &encounters, map) {
            if let Some(removed) = new_maps.insert(map.id, map) {
                panic!("Duplicate world map id {}", removed.id);
            }
        } else {
            eprintln!("Could not convert {} into a world map", map.data.name);
        }
    });

    serializable::serialize("maps", new_maps);
}

pub struct WildEncounter {}

fn into_world_map(
    mappings: &NameMappings,
    scripts: &Scripts,
    messages: &Messages,
    maps: &DashMap<String, JsonMap>,
    wild: &DashMap<String, Option<HashMap<WildType, WildEntry>>>,
    map: &JsonMap,
) -> Option<WorldMap> {
    let map_path = format!("{}/{}", PATH, map.layout.blockdata_filepath);
    let border_path = format!("{}/{}", PATH, map.layout.border_filepath);

    let map_data = attohttpc::get(map_path).send().unwrap().bytes().unwrap();
    let border_data = attohttpc::get(border_path).send().unwrap().bytes().unwrap();

    let mapdata = BinaryMap::load(
        &map_data,
        &border_data,
        map.layout.width * map.layout.height,
    )?;

    let palettes = into_palettes(
        mappings,
        &map.layout.primary_tileset,
        &map.layout.secondary_tileset,
    );

    Some(WorldMap {
        id: mappings
            .map
            .id
            .get(&map.data.id)
            .cloned()
            .unwrap_or_else(|| loc(&map.data.id)),
        name: mappings
            .map
            .name
            .get(&map.data.name)
            .unwrap_or(&map.data.name)
            // .unwrap_or_else(|| panic!("Cannot get map name mapping for {}", map.data.name))
            .clone(),
        chunk: map
            .data
            .connections
            .as_ref()
            .map(|connections| into_chunk(mappings, connections))
            .flatten(),
        warps: map
            .data
            .warps
            .iter()
            .enumerate()
            .flat_map(|(index, warp)| into_world_warp(mappings, maps, warp, index))
            .collect(),
        wild: wild.remove(&map.data.id).map(|(.., v)| v).flatten(),
        npcs: into_world_npcs(mappings, scripts, messages, &map.data.objects),
        width: map.layout.width as _,
        height: map.layout.height as _,
        palettes,
        music: into_music(mappings, &map.data.music),
        settings: Default::default(),
        tiles: mapdata.tiles,
        movements: mapdata.movements,
        border: [
            mapdata.border.tiles[0],
            mapdata.border.tiles[1],
            mapdata.border.tiles[2],
            mapdata.border.tiles[3],
        ],
        scripts: Default::default(),
    })
}

fn loc(id: &str) -> Location {
    Location {
        map: Some(tinystr16!("unnamed")),
        index: truncate_id(id),
    }
}

fn truncate_id(id: &str) -> TinyStr16 {
    let id = &id[4..];
    if id.len() >= 16 {
        format!("{}{}", &id[..12], &id[id.len() - 4..]).parse()
    } else {
        id.parse()
    }
    .unwrap()
}

fn into_chunk(mappings: &NameMappings, connections: &[JsonConnection]) -> Option<WorldChunk> {
    match connections.is_empty() {
        true => None,
        false => Some(WorldChunk {
            connections: connections
                .iter()
                .flat_map(|connection| {
                    let direction = match connection.direction.as_str() {
                        "left" => Direction::Left,
                        "right" => Direction::Right,
                        "up" => Direction::Up,
                        "down" => Direction::Down,
                        _ => unreachable!(),
                    };
                    Some((
                        direction,
                        Connection(
                            mappings
                                .map
                                .id
                                .get(&connection.map)
                                .cloned()
                                .unwrap_or_else(|| loc(&connection.map)),
                            connection.offset as _,
                        ),
                    ))
                })
                .collect(),
        }),
    }
}

fn into_world_warp(
    mappings: &NameMappings,
    maps: &DashMap<String, JsonMap>,
    warp: &JsonWarpEvent,
    index: usize,
) -> Option<(WarpId, WarpEntry)> {
    let destination = mappings
        .map
        .id
        .get(&warp.destination)
        .cloned()
        .unwrap_or_else(|| loc(&warp.destination));

    let name = format!("warp_{}", index).parse().unwrap();

    let entry = WarpEntry {
        location: BoundingBox {
            min: Coordinate {
                x: warp.x as _,
                y: warp.y as _,
            },
            max: Coordinate {
                x: warp.x as _,
                y: warp.y as _,
            },
        },
        destination: WarpDestination {
            location: destination,
            position: {
                let w = &maps
                    .get(&warp.destination)?
                    // .unwrap_or_else(|| panic!("Cannot get map at {}", warp.destination))
                    .data
                    .warps[warp.dest_warp_id as usize];
                Destination {
                    coords: Coordinate {
                        x: w.x as _,
                        y: w.y as _,
                    },
                    direction: None,
                }
            },
            transition: WarpTransition {
                move_on_exit: false,
                warp_on_tile: true,
                change_music: true,
            },
        },
    };

    Some((name, entry))
}

fn into_world_npcs(
    mappings: &NameMappings,
    scripts: &Scripts,
    messages: &Messages,
    events: &[JsonObjectEvents],
) -> Npcs {
    events
        .iter()
        .enumerate()
        .par_bridge()
        .flat_map(|(index, event)| {
            if let Some(group) = mappings.npcs.groups.get(&event.graphics_id) {
                let (movement, direction) = mappings
                    .npcs
                    .movement
                    .get(&event.movement_type)
                    .cloned()
                    .unwrap_or_default();

                let mut interact = NpcInteract::Nothing;

                if let Some(script) = scripts.get(&event.script) {
                    let script = script.value();
                    if script.commands.len() == 1 {
                        let command = &script.commands[0];
                        if &command.command == "msgbox" {
                            let message = messages.get(&command.arguments[0]).unwrap();
                            let message = message.value();
                            interact = NpcInteract::Message(message.clone());
                        }
                    }
                }

                let group = group.parse().unwrap();
                Some((
                    format!("npc_{}", index).parse().unwrap(),
                    Npc {
                        character: Character::new(
                            format!("NPC {}-{}", event.x, event.y),
                            Position {
                                coords: Coordinate {
                                    x: event.x as _,
                                    y: event.y as _,
                                },
                                direction,
                            },
                        ),
                        group,
                        movement,
                        origin: None,
                        interact,
                        trainer: None,
                    },
                ))
            } else {
                None
            }
        })
        .collect()
}

fn into_palettes(mappings: &NameMappings, primary: &str, secondary: &str) -> [PaletteId; 2] {
    let primary = mappings
        .palettes
        .primary
        .get(primary)
        .copied()
        .unwrap_or_else(|| {
            eprintln!("Unknown primary tileset {}", primary);
            0
        });
    let secondary = mappings
        .palettes
        .secondary
        .get(secondary)
        .copied()
        .unwrap_or_else(|| {
            eprintln!("Unknown secondary tileset {}", secondary);
            13
        });

    [primary, secondary]
}

fn into_music(mappings: &NameMappings, music: &str) -> TinyStr16 {
    mappings.music.get(music).copied().unwrap_or_else(|| {
        eprintln!("Cannot find music {}", music);
        tinystr16!("pallet")
    })
}

// #[derive(Debug, Deserialize, Default)]
// #[serde(from = "String")]
// pub struct JsonMovementType(pub NpcMovement, pub Direction);

// impl From<String> for JsonMovementType {
//     fn from(string: String) -> Self {
//         match string.as_str() {

//             _ => Default::default(),
//         }
//     }
// }

// impl JsonMap {
//     pub fn save(self) {
//         let path = std::path::Path::new(&self.name);

//         std::fs::create_dir_all(&path).unwrap();

//         let npcs = path.join("npcs");

//         std::fs::create_dir_all(&npcs).unwrap();

//         for (index, event) in self.object_events.into_iter().enumerate() {
//             match event {
//                 object_events::MapObjectType::Npc(npc) => {
//                     let npc = SerializedNpc {
//                         id: {
//                             let t = format!("npc_{}", index);
//                             t.parse::<NpcId>().unwrap()
//                         },
//                         npc: npc,
//                     };
//                     let data = ron::ser::to_string_pretty(&npc, Default::default())
//                         .unwrap()
//                         .into_bytes();
//                     std::fs::write(npcs.join(format!("{}.ron", &npc.npc.character.name)), data)
//                         .unwrap();
//                 }
//                 object_events::MapObjectType::Other => (),
//             }
//         }
//     }
// }
