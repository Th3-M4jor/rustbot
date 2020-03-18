use crate::util::*;
use chrono::prelude::*;
use serenity::{model::channel::Message, prelude::*};
use serenity::framework::standard::{
    Args, CommandResult,
    macros::command,
};
use std::sync::RwLock;
use std::sync::Arc;

use serde_json;
use std::time::Duration;

pub(crate) mod market;

const WARFRAME_URL: &'static str = "https://api.warframestat.us/pc";

//static WARFRAME_PC_DATA: RwLock<serde_json::Value> = RwLock::new(serde_json::Value::Null);

/*
lazy_static! {
    static ref WARFRAME_PC_DATA: RwLock<serde_json::Value> = RwLock::new(serde_json::Value::Null);
}
*/

//https://docs.google.com/document/d/1121cjBNN4BeZdMBGil6Qbuqse-sWpEXPpitQH5fb_Fo/edit#heading=h.yi84u2lickud
//URL for warframe market API

pub struct WarframeData {
    data: Arc<RwLock<serde_json::Value>>,
}

impl WarframeData {
    pub fn new() -> WarframeData {
        let data = Arc::new(RwLock::new(serde_json::Value::Null));
        let thread_dat = Arc::clone(&data);
        std::thread::spawn(move || {
            WarframeData::load_loop(thread_dat);
        });
        WarframeData {
            data,
        }
    }

    fn load_loop(warframe_json: Arc<RwLock<serde_json::Value>>) {
        let mut client = reqwest::blocking::Client::new();
        let mut wait_time: u64 = 120;
        loop {
            let res = WarframeData::load(&client);
            match res {
                Ok(info) => {
                    wait_time = 120;
                    let mut dat = warframe_json
                        .write()
                        .expect("warframe data poisoned, panicking");
                    *dat = info;
                }
                Err(e) => {
                    //errored, rebuilding client for safety
                    println!("{:?}", e);
                    client = reqwest::blocking::Client::new();
                    //wait for longer than two minutes, back-off when error
                    if wait_time < 960 {
                        wait_time *= 2;
                    }
                }
            }

            std::thread::sleep(Duration::from_secs(wait_time));
        }
    }

    fn load(
        client: &reqwest::blocking::Client,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let response = client.get(WARFRAME_URL).send()?;
        let text = response.text()?.replace("â€™", "'").replace("\u{FEFF}", "");
        let dat: serde_json::Value = serde_json::from_str(&text)?;
        return Ok(dat);
    }

    pub fn sortie(&self) -> Option<String> {
        let dat = self.data.read().ok()?;
        let mut to_ret = String::from("faction: ");
        let sortie = &dat["sortie"];
        let faction = sortie["faction"].as_str()?;
        to_ret.push_str(faction);
        to_ret.push('\n');
        let expires = sortie["expiry"].as_str()?;
        let expire_time = DateTime::parse_from_rfc3339(expires).ok()?.timestamp();
        let now = Utc::now().timestamp();
        to_ret.push_str("expires in: ");
        to_ret.push_str(&build_time_rem(now, expire_time));
        to_ret.push('\n');
        to_ret.push_str("mission types: ");
        for i in 0..=2 {
            let mission = sortie["variants"][i]["missionType"].as_str()?;
            to_ret.push_str(mission);
            to_ret.push_str(", ");
        }
        to_ret.pop();
        to_ret.pop();
        //pop off last ", "
        return Some(to_ret);
    }

    pub fn fissures(&self) -> Option<Vec<String>> {
        let dat = self.data.read().ok()?;
        let mut fissures = dat["fissures"].as_array()?.clone();
        fissures.sort_unstable_by(|a, b| {
            a["tierNum"]
                .as_i64()
                .unwrap_or(-1)
                .cmp(&b["tierNum"].as_i64().unwrap_or(-1))
        });
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
                mission,
                enemy,
                tier,
                build_time_rem(now, expire_time)
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

#[command]
#[aliases("fissures")]
pub(crate) fn get_fissures(ctx: &mut Context, msg: &Message, _: Args) -> CommandResult {
    let data = ctx.data.read();
    let warframe_dat = data.get::<WarframeData>().expect("no warframe data found");

    match warframe_dat.fissures() {
        Some(val) => long_say!(ctx, msg, &val, "\n"),
        None => say!(
            ctx,
            msg,
            "could not build fissures message, inform the owner"
        ),
    }
    return Ok(());
}

#[command]
#[aliases("sortie")]
pub(crate) fn get_sortie(ctx: &mut Context, msg: &Message, _: Args) -> CommandResult {
    let data = ctx.data.read();
    let warframe_dat = data.get::<WarframeData>().expect("no warframe data found");

    match warframe_dat.sortie() {
        Some(val) => say!(ctx, msg, &val),
        None => say!(ctx, msg, "could not build sortie message, inform the owner"),
    }
    return Ok(());
}

impl TypeMapKey for WarframeData {
    type Value = WarframeData;
}
