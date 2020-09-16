use serde::Deserialize;
use serenity::prelude::TypeMapKey;
use std::fs;

#[derive(Deserialize)]
pub struct BotData {
    pub token: String,
    #[serde(default)]
    pub owner: u64,
    #[serde(default)]
    pub admins: Vec<u64>,
    #[serde(default = "no_prefix")]
    pub cmd_prefix: String,
    #[serde(default)]
    pub phb: String,
    #[serde(default)]
    pub manager: String,
    #[serde(default)]
    pub chip_url: String,
    #[serde(default)]
    pub custom_chip_url: String,
    #[serde(default)]
    pub virus_url: String,
    #[serde(default)]
    pub ncp_url: String,
    #[serde(default)]
    pub load_custom_chips: bool,
}

impl BotData {
    /// constructs a new `BotData` object, panics if the config is not setup correctly
    pub fn new() -> BotData {
        let json_str = fs::read_to_string("./config.json").expect("config not found");
        serde_json::from_str::<BotData>(&json_str).expect("bad config json")
    }
}

impl TypeMapKey for BotData {
    type Value = BotData;
}

//'%' as default prefix
fn no_prefix() -> String {
    String::from("%")
}
