use serde::{Deserialize, Serialize};

pub mod object;
pub mod warp;
pub mod wild;

#[derive(Debug, Deserialize, Serialize)]
pub struct JsonMap {
    pub data: JsonMapData,
    pub layout: JsonMapLayout,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JsonMapData {
    pub id: String,
    pub name: String,
    pub layout: String,
    pub music: String,
    // pub music: String,
    pub floor_number: isize,

    pub connections: Option<Vec<JsonConnection>>,
    #[serde(rename = "object_events")]
    pub objects: Vec<object::JsonObjectEvents>,
    #[serde(rename = "warp_events")]
    pub warps: Vec<warp::JsonWarpEvent>,
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonMapLayout {
    pub id: String,
    pub name: String,
    pub width: usize,
    pub height: usize,
    // border_width: usize,
    // border_height: usize,
    pub primary_tileset: String,
    pub secondary_tileset: String,

    pub border_filepath: String,
    pub blockdata_filepath: String,

}

#[derive(Debug, Deserialize, Serialize)]
pub struct JsonConnection {
    pub map: String,
    pub offset: isize,
    pub direction: String,
}


#[derive(Debug, Deserialize)]
pub struct JsonMapLayouts {
    pub layouts: Vec<LayoutOrNone>,
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct LayoutOrNone {
    #[serde(with = "either::serde_untagged")]
    pub inner: either::Either<JsonMapLayout, Nothing>,
}

#[derive(Debug, Deserialize)]
pub struct Nothing {

}