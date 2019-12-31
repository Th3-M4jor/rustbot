use std::sync::RwLock;
use std::sync::Arc;
use chrono::prelude::*;
use crate::util::*;
use serenity::{
    model::channel::Message,
    prelude::*,
};

use serde_json;
use std::time::Duration;

pub (crate) mod market;

const WARFRAME_URL: &'static str = "https://api.warframestat.us/pc";

lazy_static! {
    static ref WARFRAME_DATA: Arc<RwLock<serde_json::Value>> = Arc::new(RwLock::new(serde_json::Value::Null));
}

//https://docs.google.com/document/d/1121cjBNN4BeZdMBGil6Qbuqse-sWpEXPpitQH5fb_Fo/edit#heading=h.yi84u2lickud
//URL for warframe market API

pub struct WarframeData {}

impl WarframeData {
    pub fn new() -> WarframeData {
        std::thread::spawn(WarframeData::load_loop);
        WarframeData {}
    }

    fn load_loop() {
        let mut client = reqwest::Client::new();
        let mut wait_time : u64 = 120;
        loop {
            let res = WarframeData::load(&client);
            match res {
                Ok(info) => {
                    wait_time = 120;
                    let mut dat = WARFRAME_DATA.write().expect("warframe data poisoned, panicking");
                    *dat = info;
                }
                Err(e) => {
                    //errored, rebuilding client for safety
                    println!("{:?}", e);
                    client = reqwest::Client::new();
                    //wait for longer than two minutes, back-off when error
                    wait_time *= 2;
                }
            }

            std::thread::sleep(Duration::from_secs(wait_time));
        }
    }

    fn load(client: &reqwest::Client) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let mut response = client.get(WARFRAME_URL).send()?;
        let text = response.text()?.replace("â€™", "'").replace("\u{FEFF}", "");
        let dat: serde_json::Value = serde_json::from_str(&text)?;
        return Ok(dat);
    }

    pub fn sortie(&self) -> Option<String> {
        let mut to_ret = String::from("faction: ");
        let dat = WARFRAME_DATA.read().expect("data was poisoned, panicking");
        let sortie = dat.get("sortie")?;
        let faction = sortie.get("faction")?.as_str()?;
        to_ret.push_str(faction);
        to_ret.push('\n');
        let expires = sortie.get("expiry")?.as_str()?;
        let expire_time = DateTime::parse_from_rfc3339(expires).ok()?.timestamp();
        let now = Utc::now().timestamp();
        to_ret.push_str("expires in: ");
        to_ret.push_str(&build_time_rem(now, expire_time));
        to_ret.push('\n');
        to_ret.push_str("mission types: ");
        for i in 0..=2 {
            let mission = sortie.get("variants")?.get(i)?.get("missionType")?.as_str()?;
            to_ret.push_str(mission);
            to_ret.push_str(", ");
        }
        to_ret.pop();
        to_ret.pop();
        //pop off last ", "
        return Some(to_ret);
    }

    pub fn fissures(&self) -> Option<Vec<String>> {
        let dat = WARFRAME_DATA.read().expect("data was poisoned, panicking");
        let mut fissures = dat["fissures"].as_array()?.clone();
        fissures.sort_unstable_by(|a, b|
            a["tierNum"].as_i64()
                .unwrap_or(-1)
                .cmp(
                    &b["tierNum"].as_i64().unwrap_or(-1)
                )
        );
        let mut to_ret: Vec<String> = vec![];
        let now = Utc::now().timestamp();
        for val in fissures {
            let mission = val["missionType"].as_str()?;
            let enemy = val["enemy"].as_str()?;
            let tier = val["tier"].as_str()?;
            let expiry_str = val["expiry"].as_str()?;
            let expire_time = DateTime::parse_from_rfc3339(expiry_str).ok()?.timestamp();

            let to_add = format!(
                "{}, {}, {}; expires in: {}",
                mission, enemy, tier, build_time_rem(now, expire_time)
            );
            to_ret.push(to_add);
        }
        if to_ret.len() > 0 {
            return Some(to_ret);
        } else {
            return None;
        }
    }
}

pub(crate) fn get_fissures(ctx: Context, msg: Message, _: &[&str]) {
    let data = ctx.data.read();
    let warframe_dat = data.get::<WarframeData>().expect("no warframe data found");

    match warframe_dat.fissures() {
        Some(val) => long_say!(ctx, msg, &val, "\n"),
        None => say!(ctx, msg, "could not build fissures message, inform the owner"),
    }
}

pub(crate) fn get_sortie(ctx: Context, msg: Message, _: &[&str]) {
    let data = ctx.data.read();
    let warframe_dat = data.get::<WarframeData>().expect("no warframe data found");

    match warframe_dat.sortie() {
        Some(val) => say!(ctx, msg, &val),
        None => say!(ctx, msg, "could not build sortie message, inform the owner"),
    }
}

impl TypeMapKey for WarframeData {
    type Value = WarframeData;
}

