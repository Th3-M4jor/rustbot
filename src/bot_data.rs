use serde::Deserialize;
use serenity::prelude::TypeMapKey;
use std::fs;

#[derive(Deserialize)]
pub struct BotData {
    pub token: String,
    pub owner: u64,
    pub admins: Vec<u64>,
    pub main_server: u64,
    pub cmd_prefix: String,
    pub phb: String,
    pub manager: String,
}

impl BotData {
    /**
    constructs a new BotData object, panics if the config is not setup correctly
    */
    pub fn new() -> BotData {
        let json_str = fs::read_to_string("./config.json").expect("config not found");
        return serde_json::from_str::<BotData>(&json_str).expect("bad config json");
    }
}

impl TypeMapKey for BotData {
    type Value = BotData;
}
