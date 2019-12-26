
#[macro_use]
extern crate lazy_static;


use std::collections::HashMap;
use std::process::exit;
use std::sync::{Arc, RwLock};

use serenity::{
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

use crate::bot_data::BotData;
use crate::library::{
    chip_library::*,
    ncp_library::*,
    virus_library::*,
};

use crate::dice::{roll, roll_stats};
use crate::util::send_long_message;
use std::fs;
use crate::library::chip_library::send_chip;

//use regex::Replacer;
#[macro_use]
mod util;
mod library;
mod bot_data;
mod dice;



type BotCommand = fn(&Context, &Message, &[&str]) -> ();

lazy_static! {

    static ref COMMANDS: HashMap<String, BotCommand> = {
        let mut cmd_map = HashMap::new();

        //need cast to BotCommand here once and rest are implicit to avoid compiler error due to type mismatch
        cmd_map.insert("chip".to_string(), send_chip as BotCommand);

        cmd_map.insert("cr".to_string(), send_virus_cr);
        cmd_map.insert("element".to_string(), send_chip_element);

        cmd_map.insert("ncp".to_string(), send_ncp);
        cmd_map.insert("ncpcolor".to_string(), send_ncp_color);

        cmd_map.insert("reload".to_string(), reload);
        cmd_map.insert("die".to_string(), check_exit);

        cmd_map.insert("skill".to_string(), send_chip_skill);
        cmd_map.insert("skilluser".to_string(), send_chip_skill);
        cmd_map.insert("skilltarget".to_string(), send_chip_skill);
        cmd_map.insert("skillcheck".to_string(), send_chip_skill);

        cmd_map.insert("roll".to_string(), roll);
        cmd_map.insert("rollstats".to_string(), roll_stats);
        cmd_map.insert("virus".to_string(), send_virus);
        cmd_map.insert("viruselement".to_string(), send_virus_element);
        cmd_map.insert("help".to_string(), send_help);
        return cmd_map;
    };

    static ref HELP: String = fs::read_to_string("./help.txt").unwrap_or("help text is missing, bug the owner".to_string());
}

struct Handler;

impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    fn message(&self, ctx: Context, msg: Message) {
        let mut args: Vec<&str>;
        let new_first;
        {
            let data = ctx.data.read();
            let config = data.get::<BotData>().expect("no config found");
            if !msg.content.starts_with(&config.cmd_prefix) {
                return;
            }
            args = msg.content.split(" ").collect();
            new_first = args[0].replacen(&config.cmd_prefix, "", 1);
            args[0] = new_first.as_str();
        }
        //get the command from a jump table
        let cmd_res = COMMANDS.get(&args[0].to_lowercase());
        match cmd_res {
            Some(cmd) => cmd(&ctx, &msg, &args),
            None => send_chip(&ctx, &msg, &args),
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    fn ready(&self, ctx: Context, ready: Ready) {
        let data = ctx.data.read();
        let config = data.get::<BotData>().expect("no bot data, panicking");
        println!("{} is connected!", ready.user.name);
        let guild = serenity::model::guild::Guild::get(
            &ctx, config.main_server,
        ).expect("could not find main server");
        let owner = guild.member(
            &ctx, config.owner,
        ).expect("could not grab owner");
        let owner_user = owner.user.read();
        owner_user.dm(&ctx, |m| {
            m.content("logged in, and ready");
            return m;
        }).expect("could not dm owner");
    }
}

fn check_exit(ctx: &Context, msg: &Message, _: &[&str]) {
    let data = ctx.data.read();
    let config = data.get::<BotData>().expect("config not found");

    if msg.author.id == config.owner {
        ctx.invisible();
        ctx.shard.shutdown_clean();
        exit(0);
    }
}

fn reload(ctx: &Context, msg: &Message, _: &[&str]) {
    let data = ctx.data.read();
    let config = data.get::<BotData>().expect("could not get config");
    if msg.author.id != config.owner && !config.admins.contains(msg.author.id.as_u64()) {
        return;
    }
    let mut str_to_send;
    {
        let chip_library_lock = data.get::<ChipLibrary>().expect("chip library not found");
        let mut chip_library = chip_library_lock.write().expect("chip library was poisoned, panicking");
        let chip_reload_res = chip_library.load_chips();
        //let str_to_send;
        match chip_reload_res {
            Ok(s) => str_to_send = format!("{} chips loaded\n", s),
            Err(e) => str_to_send = format!("{}\n", e.to_string()),
        }
    }
    {
        let ncp_library_lock = data.get::<NCPLibrary>().expect("ncp library not found");
        let mut ncp_library = ncp_library_lock.write().expect("chip library was poisoned, panicking");
        let count = ncp_library.load_programs();
        //say!(ctx, msg, format!("{} NCPs loaded", count));
        str_to_send.push_str(&format!("{} NCPs loaded\n", count));
    }
    {
        let virus_library_lock = data.get::<VirusLibrary>().expect("virus library not found");
        let mut virus_library = virus_library_lock.write().expect("virus library was poisoned, panicking");
        match virus_library.load_viruses() {
            Ok(s) => str_to_send.push_str(&format!("{} viruses were loaded\n", s)),
            Err(e) => str_to_send.push_str(&format!("{}", e.to_string())),
        }
    }
    say!(ctx, msg, str_to_send);
}

fn send_help(ctx: &Context, msg: &Message, _: &[&str]) {
    say!(ctx, msg, format!("```{}```", *HELP));
}




fn main() {

    let chip_library_mutex = Arc::new(RwLock::new(ChipLibrary::new()));
    let ncp_library_mutex = Arc::new(RwLock::new(NCPLibrary::new()));
    let virus_library_mutex = Arc::new(RwLock::new(VirusLibrary::new()));
    //let mut chip_library = ChipLibrary::new();

    {
        let mut chip_library = chip_library_mutex.write().unwrap();
        match chip_library.load_chips() {
            Ok(s) => {
                println!("{} chips were loaded", s);
            }
            Err(e) => {
                println!("{}", e.to_string());
            }
        }
        let mut ncp_library = ncp_library_mutex.write().unwrap();
        let ncp_count = ncp_library.load_programs();
        println!("{} programs loaded", ncp_count);
        let mut virus_library = virus_library_mutex.write().unwrap();
        match virus_library.load_viruses() {
            Ok(s) => println!("{} viruses were loaded", s),
            Err(e) => println!("{}", e.to_string()),
        }
    }

    let config = BotData::new();

    let mut client = Client::new(&config.token, Handler).expect("Err creating client");
    // set scope to ensure that lock is released immediately
    {
        let mut data = client.data.write();
        data.insert::<ChipLibrary>(chip_library_mutex);
        data.insert::<NCPLibrary>(ncp_library_mutex);
        data.insert::<VirusLibrary>(virus_library_mutex);
        data.insert::<BotData>(config);
    }
    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}

