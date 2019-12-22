#[macro_use]
extern crate lazy_static;

use serde_json;
use serde::Deserialize;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct BotData {
    pub token: String,
    pub owner: u64,
    pub admins: Vec<u64>,
    pub main_server: u64,
}

use std::fs;

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

lazy_static! {
    static ref BOT_CONFIG: BotData = {
        let json_str = fs::read_to_string("./config.json").expect("config not found");
        return serde_json::from_str(&json_str).expect("bad config json");
    };

    static ref COMMANDS: HashMap<String, BotCommand> = {
        let mut cmd_map = HashMap::new();

        //need cast to BotCommand here once and rest are implicit to avoid compiler error due to type mismatch
        cmd_map.insert("chip".to_string(), send_chip as BotCommand);
        cmd_map.insert("die".to_string(), check_exit);
        cmd_map.insert("skill".to_string(), send_skill);
        cmd_map.insert("skilluser".to_string(), send_skill);
        cmd_map.insert("skilltarget".to_string(), send_skill);
        cmd_map.insert("skillcheck".to_string(), send_skill);
        cmd_map.insert("element".to_string(), send_element);
        cmd_map.insert("reload".to_string(), reload);
        return cmd_map;
    };
}

struct Handler;

impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    fn message(&self, ctx: Context, msg: Message) {
        if !msg.content.starts_with("%") {
            return;
        }
        let mut args: Vec<&str> = msg.content.split(" ").collect();
        let new_first = args[0].replacen("%", "", 1);
        args[0] = new_first.as_str();
        //args[0] = args[0].replacen("%", "", 1).as_str();
        let cmd_res = COMMANDS.get(&new_first.to_lowercase());
        match cmd_res {
            Some(cmd) => cmd(&ctx,&msg,&args),
            None => send_chip(&ctx, &msg, &args),
        }
        /*
        match new_first.to_lowercase().as_str() {
            "chip" => send_chip(&ctx, &msg, &args),
            "die" => check_exit(&ctx, &msg, &args),
            "skill" => send_skill(&ctx, &msg, &args),
            "skilluser" => send_skill(&ctx, &msg, &args),
            "skilltarget" => send_skill(&ctx, &msg, &args),
            "skillcheck" => send_skill(&ctx, &msg, &args),
            "element" => send_element(&ctx, &msg, &args),
            "reload" => reload(&ctx, &msg, &args),
            _ => send_chip(&ctx, &msg, &args),
        }
        */
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        let guild = serenity::model::guild::Guild::get(
            &ctx, BOT_CONFIG.main_server,
        ).expect("could not find bentTest");
        let owner = guild.member(
            &ctx, BOT_CONFIG.owner,
        ).expect("could not grab major");
        let major = owner.user.read();
        major.dm(&ctx, |m| {
            m.content("logged in, and ready");
            return m;
        }).expect("could not dm owner");
    }
}
    fn send_chip(ctx: &Context, msg: &Message, args: &Vec<&str>) {
        let to_get;
        if args.len() < 2 {
            to_get = args[0];
        } else {
            to_get = args[1];
        }
        let data = ctx.data.read();
        let library_lock = data.get::<ChipLibrary>().expect("chip library not found");
        let library = library_lock.read().expect("library was poisoned, panicking");
        //let library = locked_library.read().expect("library was poisoned");

        let chip = library.get(to_get);
        if chip.is_some() {
            if let Err(why) = msg.channel_id.say(&ctx.http, format!("{}", chip.unwrap())) {
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
            }
        }
        let to_send: String = chip_search.join(", ");
        if let Err(why) = msg.channel_id.say(&ctx.http, format!("Did you mean: {}", to_send)) {
            println!("Could not send message: {:?}", why);
        }
    }

    fn check_exit(_: &Context, msg: &Message, _: &Vec<&str>) {
        if msg.author.id == BOT_CONFIG.owner {
            exit(0);
        }
    }

    fn send_skill(ctx: &Context, msg: &Message, args: &Vec<&str>) {
        if args.len() < 2 {
            if let Err(why) = msg.channel_id.say(&ctx.http, "you must provide a skill") {
                println!("Could not send message: {:?}", why);
            }
            return;
        }
        let data = ctx.data.read();
        let library_lock = data.get::<ChipLibrary>().expect("chip library not found");
        let library = library_lock.read().expect("chip library poisoned, panicking");
        let skill_res;// = library.search_skill(args[1]);
        match args[0].to_lowercase().as_str() {
            "skill" => skill_res = library.search_skill(args[1]),
            "skilluser" => skill_res = library.search_skill_user(args[1]),
            "skilltarget" => skill_res = library.search_skill_target(args[1]),
            "skillcheck" => skill_res = library.search_skill_check(args[1]),
            _ => panic!("should not have gotten here"),
        }
        if skill_res.is_some() {
            if let Err(why) = send_string_vec(&ctx, &msg, &skill_res.unwrap()) {
                println!("Could not send message: {:?}", why);
            }
            return;
        }
        if let Err(why) = msg.channel_id.say(&ctx.http, "nothing matched your search") {
            println!("Could not send message: {:?}", why);
        }
    }
    fn send_element(ctx: &Context, msg: &Message, args: &Vec<&str>) {
        if args.len() < 2 {
            if let Err(why) = msg.channel_id.say(&ctx.http, "you must provide an element") {
                println!("Could not send message: {:?}", why);
            }
            return;
        }
        let data = ctx.data.read();
        let library_lock = data.get::<ChipLibrary>().expect("chip library not found");
        let library = library_lock.read().expect("chip library poisoned, panicking");
        let elem_res = library.search_element(args[1]);
        if elem_res.is_some() {
            if let Err(why) = send_string_vec(&ctx, &msg, &elem_res.unwrap()) {
                println!("Could not send message: {:?}", why);
            }
        }
    }

    fn reload(ctx: &Context, msg: &Message, _: &Vec<&str>) {
        if msg.author.id != BOT_CONFIG.owner && !BOT_CONFIG.admins.contains(msg.author.id.as_u64()) {
            return;
        }
        let data = ctx.data.read();
        let chip_library_lock = data.get::<ChipLibrary>().expect("chip library not found");
        let mut chip_library = chip_library_lock.write().expect("chip library was poisoned, panicking");
        let chip_reload_res = chip_library.load_chips();
        let str_to_send;
        match chip_reload_res {
            Ok(s) => str_to_send = format!("{} chips loaded", s),
            Err(e) => str_to_send = format!("{}", e.to_string()),
        }
        if let Err(why) = msg.channel_id.say(&ctx.http, str_to_send) {
            println!("Could not send message: {:?}", why);
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

type BotCommand = fn(&Context, &Message, &Vec<&str>) -> ();

fn main() {
    /*
    let mut cmd_map: HashMap<String, BotCommand> = HashMap::new();
    cmd_map.insert("chip".to_string(), send_chip);
    cmd_map.insert("die".to_string(), check_exit);
    cmd_map.insert("skill".to_string(), send_skill);
    cmd_map.insert("skilluser".to_string(), send_skill);
    cmd_map.insert("skilltarget".to_string(), send_skill);
    cmd_map.insert("skillcheck".to_string(), send_skill);
    cmd_map.insert("element".to_string(), send_element);
    cmd_map.insert("reload".to_string(), reload);
    */
    let chip_library_mutex = Arc::new(RwLock::new(ChipLibrary::new()));
    //let mut chip_library = ChipLibrary::new();

    {
        let mut chip_library = chip_library_mutex.write().unwrap();
        let chip_load_res = chip_library.load_chips();
        match chip_load_res {
            Ok(s) => {
                println!("{} chips were loaded", s);
            }
            Err(e) => {
                println!("{}", e.to_string());
            }
        }
    }

    //let token = fs::read_to_string("./token.txt").expect("token not loaded");
    let mut client = Client::new(&BOT_CONFIG.token, Handler).expect("Err creating client");
    // set scope to ensure that lock is released immediately
    {
        let mut data = client.data.write();
        data.insert::<ChipLibrary>(chip_library_mutex);
    }
    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}

