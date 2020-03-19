use serenity::{model::channel::Message, prelude::*};
use serenity::framework::standard::{macros::command, Args, CommandResult};
use std::sync::RwLock;
use serde_json;
use std::error::Error;

pub struct Blights {
    values: serde_json::Value,
}

impl Blights {
    pub fn new() -> Blights {
        Blights {
            values: serde_json::Value::Null,
        }
    }

    pub fn load(&mut self) -> Result<(), Box<dyn Error>> {
        let blights = std::fs::read_to_string("./blights.json")?;
        self.values = serde_json::from_str(&blights)?;
        return Ok(());
    }

    pub fn get(&self, elem: &str) -> Option<&str> {
        return self.values.as_object()?.get(&elem.to_lowercase())?.as_str();
    }

}

impl TypeMapKey for Blights {
    type Value = RwLock<Blights>;
}

#[command]
#[aliases("blight")]
pub(crate) fn get_blight(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult{
    if args.len() < 1 {
        say!(ctx, msg, "you must provide an element");
        return Ok(());
    }
    let data = ctx.data.read();
    let blight_lock = data.get::<Blights>().expect("blights not found");
    let blights = blight_lock
        .read()
        .expect("blights poisoned, panicking");
    let res = blights.get(args.current().unwrap());//.unwrap_or("There is no blight with that element, perhaps you spelled it wrong?");
    let to_send = match res {
        Some(val) => format!("```{}```", val),
        None => String::from("There is no blight with that element, perhaps you spelled it wrong?"),
    };
    say!(ctx, msg, to_send);
    return Ok(());
}