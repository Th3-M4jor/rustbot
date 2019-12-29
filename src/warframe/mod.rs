use std::sync::RwLock;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use chrono::prelude::*;
use reqwest::Client;

use serenity::{
    model::channel::Message,
    prelude::*,
};

use serde_json;

const WARFRAME_URL: &'static str = "https://api.warframestat.us/pc";

pub struct WarframeData {
    data: Arc<RwLock<serde_json::Value>>,
    next_update: DateTime<Utc>,
    client: Arc<RwLock<Client>>,
    is_updating: Arc<AtomicBool>,
}

impl WarframeData {
    pub fn new() -> WarframeData {
        WarframeData {
            data : Arc::new(RwLock::new(serde_json::Value::Null)),
            next_update: Utc::now(),
            client: Arc::new(RwLock::new(Client::new())),
            is_updating: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn load(&mut self) -> Result<(), serde_json::Error> {
        if self.is_updating.load(Ordering::Acquire) == true {
            return Ok(());
        }
        let res = self.is_updating.compare_exchange(false, true,Ordering::Acquire, Ordering::Relaxed);
        if res.is_err() {
            return Ok(());
        }
        let client = self.client.write().unwrap();
        let text = client.get(WARFRAME_URL).send()
            .expect("no request result").text().expect("no response text")
            .replace("â€™", "'").replace("\u{FEFF}", "");
        let mut dat = self.data.write().expect("data was poisoned, panicking");
        *dat = serde_json::from_str(&text)?;
        self.next_update = Utc::now() + chrono::Duration::minutes(15);
        self.is_updating.store(false, Ordering::Relaxed);
        return Ok(());
    }

    pub fn sortie(&self) -> Option<String> {
        let mut to_ret = String::from("faction: ");
        let dat = self.data.read().expect("data was poisoned, panicking");
        let sortie = dat.get("sortie")?;
        let faction = sortie.get("faction")?.as_str()?;
        to_ret.push_str(faction);
        to_ret.push('\n');
        let expires = sortie.get("expiry")?.as_str()?;
        let expire_time = DateTime::parse_from_rfc3339(expires).ok()?;
        to_ret.push_str("expires: ");
        to_ret.push_str(&expire_time.to_rfc2822());
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

    pub fn needs_update(&self) -> bool {
        if self.is_updating.load(Ordering::Relaxed) == true {
            return false;
        }
        let now = Utc::now();
        return self.next_update.cmp(&now) == std::cmp::Ordering::Greater;
    }

}

pub (crate) fn get_sortie(ctx: &Context, msg: &Message, _: &[&str]) {
    let data = ctx.data.read();
    let warframe_dat_lock = data.get::<WarframeData>().expect("no warframe data found");
    {
        let warframe_dat = warframe_dat_lock.read().expect("warframe data poisoned, panicking");
        if !warframe_dat.needs_update() {
            match warframe_dat.sortie() {
                Some(val) => say!(ctx, msg, &val),
                None => say!(ctx, msg, "something didn't work, inform the owner"),
            }
            return;
        }
    }
    let mut warframe_dat = warframe_dat_lock.write().expect("warframe data poisoned, panicking");
    if warframe_dat.load().is_err() {
        say!(ctx, msg, "Internal error loading warframe data, inform the owner");
        return;
    }
    match warframe_dat.sortie() {
        Some(val) => say!(ctx, msg, &val),
        None => say!(ctx, msg, "could not build sortie message, inform the owner"),
    }
}

impl TypeMapKey for WarframeData {
    type Value = Arc<RwLock<WarframeData>>;
}

