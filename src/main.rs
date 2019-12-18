#[macro_use]
extern crate lazy_static;

use std::env;
use std::fs;

use serenity::{
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use crate::library::ChipLibrary;
use std::process::exit;
use serenity::model::gateway::Activity;
use serenity::model::user::OnlineStatus;

mod battlechip;
mod library;

struct Handler;

impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            // Sending a message can fail, due to a network error, an
            // authentication error, or lack of permissions to post in the
            // channel, so log to stdout when some error happens, with a
            // description of it.
            let chip = ctx.data.read().get::<ChipLibrary>().expect("library not found");
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!") {
                println!("Error sending message: {:?}", why);
            }
        } else if msg.content == "!die" {
            ctx.shard.set_status(OnlineStatus::Invisible);
            exit(0);
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

fn main() {
    let mut chip_library = ChipLibrary::new();
    let load_res = chip_library.load_chips();
    match load_res {
        Ok(s) => {
            println!("{} chips were loaded", s);
        }
        Err(e) => {
            println!("{}", e.to_string());
        }
    }

    let token = fs::read_to_string("./token.txt").expect("token not loaded");
    let mut client = Client::new(&token, Handler).expect("Err creating client");
    {
        let mut data = client.data.write();
        data.insert::<ChipLibrary>(chip_library);
    }
    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }



}

