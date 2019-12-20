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
//use std::sync::Arc;
//use serenity::model::gateway::Activity;

mod battlechip;
mod library;
mod distance;

struct Handler;

impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    fn message(&self, ctx: Context, msg: Message) {
        if msg.content.starts_with("%") {
            let mut args : Vec<&str> = msg.content.split(" ").collect();
            let new_first = args[0].replacen("%", "", 1);
            args[0] = new_first.as_str();
            //args[0] = args[0].replacen("%", "", 1).as_str();
            match new_first.to_lowercase().as_str() {
                "chip" => Handler::send_chip(&ctx, &msg, &args),
                "die" => Handler::check_exit(&ctx, &msg, &args),
                "skill" => Handler::send_skill(&ctx, &msg, &args),
                _ => Handler::send_chip(&ctx, &msg, &args),
            }
            //Handler::send_chip(&ctx, &msg, &args);
        } else if msg.content == "!ping" {
            // Sending a message can fail, due to a network error, an
            // authentication error, or lack of permissions to post in the
            // channel, so log to stdout when some error happens, with a
            // description of it.
            /*
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
            */
            return;
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
        major.dm(&ctx, |m| {
            m.content("logged in, and ready");
            return m;
        }).expect("could not dm owner");
    }
}

impl Handler {
    fn send_chip(ctx: &Context, msg: &Message, args: &Vec<&str>) {
        let to_get;
        if args.len() < 2 {
            to_get = args[0];
        } else {
            to_get = args[1];
        }
        let data = ctx.data.read();
        let library = data.get::<ChipLibrary>().expect("library not found");

        //let library = locked_library.read().expect("library was poisoned");

        let chip = library.get(to_get);
        if chip.is_some() {

            if let Err(why) = msg.reply(&ctx.http, chip.unwrap()) {
                println!("Could not send message: {:?}", why);
            }
            return;
        }
        //else no chip

        //let chip_search = library.contains(to_get);
        let chip_search;
        match library.name_contains(to_get) {
            Some(t) => chip_search = t,
            None => {
                chip_search = library.distance(to_get);
            },
        }
        let to_send: String = chip_search.join(", ");
        if let Err(why) = msg.reply(&ctx.http, format!("Did you mean: {}", to_send)) {
            println!("Could not send message: {:?}", why);
        }

    }
    fn check_exit( _: &Context, msg: &Message, _: &Vec<&str>) {
        if msg.author.id == 254394113934229504 {
            exit(0);
        }
    }
    fn send_skill(ctx: &Context, msg: &Message, args: &Vec<&str>) {
        if args.len() < 2 {
            if let Err(why) = msg.reply(&ctx.http, "you must provide a skill") {
                println!("Could not send message: {:?}", why);
            }
            return;
        }
        let data = ctx.data.read();
        let library = data.get::<ChipLibrary>().expect("library not found");
        let skill_res = library.search_skill(args[1]);
        if skill_res.is_some() {
            if let Err(why) = Handler::send_string_vec(&ctx, &msg, &skill_res.unwrap()){
                println!("Could not send message: {:?}", why);
            }
        }

    }

    fn send_string_vec(ctx: &Context, msg: &Message, to_send: &Vec<String>) -> serenity::Result<Message> {
        let mut reply = String::new();
        for val in to_send {

            //a single message cannot be greater than 2000 chars
            if reply.len() + val.len() > 1950 {
                msg.channel_id.say(&ctx.http, &reply)?;
                reply.clear();
            }
            reply.push_str(val.as_str());
            reply.push_str(", ");
        }
        //remove last ", "
        reply.pop();
        reply.pop();
        return msg.channel_id.say(&ctx.http, &reply);
    }

}

fn main() {
    //let chip_library_mutex = RwLock::new(ChipLibrary::new());
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

