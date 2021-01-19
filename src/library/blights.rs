use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    prelude::*,
};
use std::error::Error;
use tokio::sync::RwLock;
use serde_json::{Value, json};


pub trait StatusLike {
    fn get_values(&self) -> &Value;

    fn to_slash_opts(&self) -> Value {

        let obj = match self.get_values().as_object() {
            Some(list) => list,
            None => return serde_json::Value::Null,
        };

        let mut list = obj.keys().collect::<Vec<&String>>();

        list.sort_unstable();

        let res = list.iter().map(|v| {
            json!({
                "name": v,
                "value": v,
            })
        }).collect::<Vec<Value>>();

        Value::Array(res)
    }
}


pub struct Blights {
    values: Value,
}

impl StatusLike for Blights {
    fn get_values(&self) -> &Value {
        &self.values
    }
}

impl Blights {
    pub async fn import() -> Result<RwLock<Blights>, Box<dyn Error>> {
        
        let mut to_ret = Blights {
            values: serde_json::Value::Null,
        };
        
        to_ret.load().await?;

        Ok(RwLock::new(to_ret))

    }

    pub async fn load(&mut self) -> Result<(), Box<dyn Error>> {
        let blights = tokio::fs::read_to_string("./blights.json").await?;
        self.values = serde_json::from_str(&blights)?;
        Ok(())
    }

    pub fn get(&self, elem: &str) -> Option<&str> {
        self.values.as_object()?.get(&elem.to_lowercase())?.as_str()
    }
}

impl TypeMapKey for Blights {
    type Value = RwLock<Blights>;
}

#[command("blight")]
/// Get info on what a blight from an element does
#[example = "Fire"]
pub(crate) async fn get_blight(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
        reply!(ctx, msg, "you must provide an element");
        return Ok(());
    }
    let data = ctx.data.read().await;
    let blight_lock = data.get::<Blights>().expect("blights not found");
    let blights = blight_lock.read().await;
    let res = blights.get(args.current().unwrap()); //.unwrap_or("There is no blight with that element, perhaps you spelled it wrong?");
    let to_send = match res {
        Some(val) => format!("```{}```", val),
        None => String::from("There is no blight with that element, perhaps you spelled it wrong?"),
    };
    reply!(ctx, msg, to_send);
    Ok(())
}

pub struct Statuses {
    values: Value,
}

impl StatusLike for Statuses {
    fn get_values(&self) -> &Value {
        &self.values
    }
}

impl Statuses {
    pub async fn import() -> Result<RwLock<Statuses>, Box<dyn Error>> {
        
        let mut to_ret = Statuses {
            values: Value::Null,
        };

        to_ret.load().await?;

        Ok(RwLock::new(to_ret))

    }

    pub async fn load(&mut self) -> Result<(), Box<dyn Error>> {
        let blights = tokio::fs::read_to_string("./statuses.json").await?;
        self.values = serde_json::from_str(&blights)?;
        Ok(())
    }

    pub fn get(&self, status: &str) -> Option<&str> {
        self.values.as_object()?.get(&status.to_lowercase())?.as_str()
    }

}

impl TypeMapKey for Statuses {
    type Value = RwLock<Statuses>;
}

#[command("status")]
/// Get info on what a status means
#[example = "Blind"]
pub(crate) async fn get_status(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
        reply!(ctx, msg, "you must provide a status");
        return Ok(());
    }

    let data = ctx.data.read().await;
    let status_lock = data.get::<Statuses>().expect("statuses not found");
    let statuses = status_lock.read().await;
    let res = statuses.get(args.current().unwrap()); //.unwrap_or("There is no blight with that element, perhaps you spelled it wrong?");
    let to_send = match res {
        Some(val) => format!("```{}```", val),
        None => String::from("There is no status with that name, perhaps you spelled it wrong?"),
    };

    reply!(ctx, msg, to_send);

    Ok(())
}

pub struct Panels {
    values: serde_json::Value,
}

impl StatusLike for Panels {
    fn get_values(&self) -> &Value {
        &self.values
    }
}

impl Panels {
    pub async fn import() -> Result<RwLock<Panels>, Box<dyn Error>> {
        let mut to_ret = Panels {
            values: Value::Null,
        };

        to_ret.load().await?;

        Ok(RwLock::new(to_ret))

    }

    pub async fn load(&mut self) -> Result<(), Box<dyn Error>> {
        let blights = tokio::fs::read_to_string("./panels.json").await?;
        self.values = serde_json::from_str(&blights)?;
        Ok(())
    }

    pub fn get(&self, kind: &str) -> Option<&str> {
        self.values.as_object()?.get(&kind.to_lowercase())?.as_str()
    }

}

impl TypeMapKey for Panels {
    type Value = RwLock<Panels>;
}

#[command("panel")]
#[aliases("terrain")]
/// Get info on what a panel type means
#[example = "lava"]
pub(crate) async fn get_panels(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
        reply!(ctx, msg, "you must provide a panel type");
        return Ok(());
    }

    let data = ctx.data.read().await;
    let panel_lock = data.get::<Panels>().expect("panels not found");
    let panels = panel_lock.read().await;
    let res = panels.get(args.current().unwrap()); //.unwrap_or("There is no blight with that element, perhaps you spelled it wrong?");
    let to_send = match res {
        Some(val) => format!("```{}```", val),
        None => String::from("There is no panel with that name, perhaps you spelled it wrong?"),
    };
    reply!(ctx, msg, to_send);

    Ok(())
}