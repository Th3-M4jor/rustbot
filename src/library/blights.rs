use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    prelude::*,
};
use std::error::Error;
use tokio::sync::RwLock;

pub struct Blights {
    values: serde_json::Value,
}

impl Blights {
    pub fn new() -> Blights {
        Blights {
            values: serde_json::Value::Null,
        }
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
#[description("Get info on what a blight from an element does")]
#[example = "Fire"]
pub(crate) async fn get_blight(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
        say!(ctx, msg, "you must provide an element");
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
    say!(ctx, msg, to_send);
    return Ok(());
}
