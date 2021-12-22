use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct JsonObjectEvents {
    pub graphics_id: String,
    pub x: isize,
    pub y: isize,
    pub elevation: isize,
    pub movement_type: String,
}
