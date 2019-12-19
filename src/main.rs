#[macro_use]
extern crate lazy_static;


use std::fs;
use serenity::model::user::OnlineStatus;

use serenity::{
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use crate::library::ChipLibrary;
use std::process::exit;
//use serenity::model::gateway::Activity;

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
        if msg.content.starts_with("%") {
            let args : Vec<&str> = msg.content.split(" ").collect();

        } else if msg.content == "!ping" {
            // Sending a message can fail, due to a network error, an
            // authentication error, or lack of permissions to post in the
            // channel, so log to stdout when some error happens, with a
            // description of it.
            let data = ctx.data.read();
            let library = data.get::<ChipLibrary>().expect("library not found");
            let chip = library.get("AirHoc");

            let to_send = match chip {
                Some(a_chip) => format!("```{}```",a_chip.All),
                _ => format!("{}", "no chip found"),
            };

            if let Err(why) = msg.channel_id.say(&ctx.http, to_send) {
                println!("Error sending message: {:?}", why);
            }
        } else if msg.content == "!die" {
            ctx.set_presence(None, OnlineStatus::Invisible);
            exit(0);
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        let guild = serenity::model::guild::Guild::get(&ctx,434770085761253377).expect("could not find bentTest");
        let owner = guild.member(&ctx, 254394113934229504).expect("could not grab major");//.user.get_mut();
        let major = owner.user.read();
        let res = major.dm(&ctx, |m| {
            m.content("logged in, and ready");
            return m;
        }).expect("could not dm owner");
    }
}

impl Handler {
    fn send_chip_as_arg(&self, ctx: Context, msg: Message, args: Vec<&str>) {
        if args.len() < 2 {
            let res = msg.channel_id.say(&ctx, "Must specify argument");
            if res.is_err() {
                println!("could not send a message to a channel");
            }
            return;
        }
        let data = ctx.data.read();
        let library = data.get::<ChipLibrary>().expect("library not found");
        let chip = library.get(args[1]);

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

