use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct JsonObjectEvents {
    pub graphics_id: String,
    pub x: isize,
    pub y: isize,
    pub elevation: isize,
    pub movement_type: String,
    pub movement_range_x: u8,
    pub movement_range_y: u8,
    pub trainer_type: String,
    pub trainer_sight_or_berry_tree_id: String,
    pub script: String,
    pub flag: String,
}
