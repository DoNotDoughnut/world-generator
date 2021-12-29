use std::path::Path;

use firecore_world_builder::world::{map::WorldMap, positions::Location};
use rayon::iter::{ParallelBridge, ParallelIterator};

pub fn serialize<P: AsRef<Path>>(root: P, maps: &dashmap::DashMap<Location, WorldMap>) {
    let root = root.as_ref();

    let files = root.join("files");

    let copies = root.join("copies");

    std::fs::create_dir_all(&files).unwrap();

    std::fs::create_dir_all(&copies).unwrap();

    maps.iter().par_bridge().for_each(|r| {
        let location = r.key();
        let map = r.value();
        let data = bincode::serialize(&map).unwrap();

        let path = match location.map {
            Some(map) => format!("{}-{}.world", map, location.index),
            None => format!("{}.world", location.index),
        };

        let file = files.join(&path);

        std::fs::write(file, &data).unwrap();

        let copy = copies.join(&path);

        let str = ron::ser::to_string_pretty(&map, Default::default()).unwrap();

        std::fs::write(copy, str.as_bytes()).unwrap();


        // std::fs::create_dir_all(&path).unwrap();

        // {
        //     let data = MapConfig {
        //         identifier: location.into(),
        //         name: map.name,
        //         chunk: map
        //             .chunk
        //             .map(|c| {
        //                 c.connections
        //                     .into_iter()
        //                     .map(|(a, b)| (a, b.into()))
        //                     .collect()
        //             })
        //             .unwrap_or_default(),
        //         map: "map.bin".into(),
        //         border: "border.bin".into(),
        //         width: map.width as _,
        //         height: map.height as _,
        //         palettes: map.palettes,
        //         music: map.music,
        //         settings: map.settings,
        //     };

        //     let data = ron::ser::to_string_pretty(&data, Default::default())
        //         .unwrap()
        //         .into_bytes();

        //     let file = path.join(format!("{}.ron", location.index));
        //     std::fs::write(file, data).unwrap()
        // }

        // let warps = path.join("warps");

        // std::fs::create_dir_all(&warps).unwrap();

        // for (id, warp) in map.warps {
        //     let data = ron::ser::to_string_pretty(&warp, Default::default())
        //         .unwrap()
        //         .into_bytes();

        //     std::fs::write(warps.join(format!("{}.ron", id)), &data).unwrap();
        // }

        // let npcs = path.join("npcs");

        // std::fs::create_dir_all(&npcs).unwrap();

        // for (id, npc) in map.npcs {
        //     let data = SerializedNpc { id, npc };

        //     let data = ron::ser::to_string_pretty(&data, Default::default())
        //         .unwrap()
        //         .into_bytes();

        //     std::fs::write(npcs.join(format!("{}.ron", id)), data).unwrap();
        // }

        // let npcs = path.join("npcs");

        // std::fs::create_dir(&npcs).unwrap();
    });
}
