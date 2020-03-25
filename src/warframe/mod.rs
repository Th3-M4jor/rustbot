use crate::util::*;
use chrono::prelude::*;
use serenity::{model::channel::Message, prelude::*};
use serenity::framework::standard::{
    Args, CommandResult,
    macros::*,
};
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::sync::Arc;

use serde_json;
use std::time::Duration;
use market::*;

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
        WarframeData {
            data,
        }
    }
    

    async fn load(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut dat : RwLockWriteGuard<serde_json::Value> = self.data.write().await;
        let response = reqwest::get(WARFRAME_URL).await?; //client.get(WARFRAME_URL).send()?;
        let text = response.text().await?.replace("â€™", "'").replace("\u{FEFF}", "");
        *dat = serde_json::from_str(&text)?;
        drop(dat);
        let dat_lock_clone = Arc::clone(&self.data);
        tokio::spawn(async move {
            tokio::time::delay_for(Duration::from_secs(300)).await;
            #[cfg(debug_assertions)]
            {
                println!("Removing cached Warframe data");
            }
            let mut dat : RwLockWriteGuard<serde_json::Value> = dat_lock_clone.write().await;
            *dat = serde_json::Value::Null;
        });
        return Ok(());
    }

    async fn try_reload(&self) -> Result<bool, Box<dyn std::error::Error>> {
        
        let data : RwLockReadGuard<serde_json::Value> = self.data.read().await;
        if !data.is_null() {
            return Ok(false);
        }
        
        drop(data);
        self.load().await?;
        return Ok(true);
    }

    pub async fn sortie(&self) -> Option<String> {
        self.try_reload().await.ok()?;
        let dat : RwLockReadGuard<serde_json::Value> = self.data.read().await;
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

    pub async fn fissures(&self) -> Option<Vec<String>> {
        self.try_reload().await.ok()?;
        let dat : RwLockReadGuard<serde_json::Value> = self.data.read().await;
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
            if expire_time < 0 {
                continue;
            }
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


#[group]
#[prefixes("w", "warframe")]
#[commands(get_sortie, get_fissures, market)]
#[description("A group of commands related to Warframe (PC)")]
struct Warframe;

#[command("fissures")]
#[description = "Get info about the current Warframe fissures (PC)"]
pub(crate) async fn get_fissures(ctx: &mut Context, msg: &Message, _: Args) -> CommandResult {
    if let Err(_) = msg.channel_id.broadcast_typing(&ctx.http).await {
        println!("could not broadcast typing, not reloading");
        return Ok(());
    }
    let data = ctx.data.read().await;
    let warframe_dat = data.get::<WarframeData>().expect("no warframe data found");

    match warframe_dat.fissures().await {
        Some(val) => long_say!(ctx, msg, &val, "\n"),
        None => say!(
            ctx,
            msg,
            "could not build fissures message, inform the owner"
        ),
    }
    return Ok(());
}

#[command("sortie")]
#[description = "Get info about the current Warframe sortie (PC)"]
pub(crate) async fn get_sortie(ctx: &mut Context, msg: &Message, _: Args) -> CommandResult {
    if let Err(_) = msg.channel_id.broadcast_typing(&ctx.http).await {
        println!("could not broadcast typing, not reloading");
        return Ok(());
    }
    let data = ctx.data.read().await;
    let warframe_dat = data.get::<WarframeData>().expect("no warframe data found");

    match warframe_dat.sortie().await {
        Some(val) => say!(ctx, msg, &val),
        None => say!(ctx, msg, "could not build sortie message, inform the owner"),
    }
    return Ok(());
}

impl TypeMapKey for WarframeData {
    type Value = WarframeData;
}
