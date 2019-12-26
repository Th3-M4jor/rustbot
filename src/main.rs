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
use crate::chip_library::{ChipLibrary, Library};
use crate::ncp_library::NCPLibrary;
use crate::virus_library::VirusLibrary;
use crate::dice::DieRoll;
use std::borrow::BorrowMut;
use std::fs;
use std::ops::Deref;
//use regex::Replacer;

mod battlechip;
mod chip_library;
mod ncp_library;
mod virus_library;
mod bot_data;
mod dice;

type BotCommand = fn(&Context, &Message, &[&str]) -> ();

lazy_static! {

    static ref COMMANDS: HashMap<String, BotCommand> = {
        let mut cmd_map = HashMap::new();

        //need cast to BotCommand here once and rest are implicit to avoid compiler error due to type mismatch
        cmd_map.insert("chip".to_string(), send_chip as BotCommand);

        cmd_map.insert("cr".to_string(), get_cr);
        cmd_map.insert("die".to_string(), check_exit);
        cmd_map.insert("skill".to_string(), send_skill);
        cmd_map.insert("skilluser".to_string(), send_skill);
        cmd_map.insert("skilltarget".to_string(), send_skill);
        cmd_map.insert("skillcheck".to_string(), send_skill);
        cmd_map.insert("element".to_string(), send_element);
        cmd_map.insert("reload".to_string(), reload);
        cmd_map.insert("roll".to_string(), roll);
        cmd_map.insert("rollstats".to_string(), roll_stats);
        cmd_map.insert("ncp".to_string(), send_ncp);
        cmd_map.insert("virus".to_string(), send_virus);
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

///fn say(ctx: Context, msg: Message, say: an expression returning a string)
macro_rules! say {
    ($ctx: ident, $msg: ident, $say: expr) => {
        if let Err(why) = $msg.channel_id.say(&$ctx.http, $say) {
            println!("Could not send message: {:?}", why);
        }
    };
}

/*
///fn search(ctx: Context, msg: Message, search: &str, lib: Library)
macro_rules! search {
    ($ctx: ident, $msg: ident, $search: expr, $lib: ident) => {
        {
            let item = $lib.get($search);
            if item.is_some() {
                say!($ctx, $msg, format!("{}", item.unwrap()));
                return;
            }
            let item_search;
            match $lib.name_contains($search) {
                Some(t) => item_search = t,
                None => item_search = $lib.distance($search),
            }
            if item_search.len() == 1 {
                let found_item = $lib.get(&item_search[0]).unwrap();
                say!($ctx, $msg, format!("{}", found_item));
                return;
            }
            let to_send: String = item_search.join(", ");
            say!($ctx, $msg, format!("Did you mean: {}", to_send));
        }
    };
}
*/

fn search_lib_obj<T: Library>(ctx: &Context, msg: &Message, search: &str, lib: &T) {
    let item = lib.get(search);
    if item.is_some() {
        say!(ctx, msg, format!("{}", item.unwrap()));
        return;
    }
    let item_search;
    match lib.name_contains(search) {
        Some(t) => item_search = t,
        None => item_search = lib.distance(search),
    }
    if item_search.len() == 1 {
        let found_item = lib.get(&item_search[0]).unwrap();
        say!(ctx, msg, format!("{}", found_item));
        return;
    }
    let to_send: String = item_search.join(", ");
    say!(ctx, msg, format!("Did you mean: {}", to_send));
}

fn send_chip(ctx: &Context, msg: &Message, args: &[&str]) {
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
    //search!(ctx, msg, to_get, library);
    search_lib_obj(ctx, msg, to_get, library.deref());
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

fn send_skill(ctx: &Context, msg: &Message, args: &[&str]) {
    if args.len() < 2 {
        say!(ctx, msg, "you must provide a skill");
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
        _ => unreachable!(),
    }
    if skill_res.is_some() {
        if let Err(why) = send_string_vec(&ctx, &msg, &skill_res.unwrap()) {
            println!("Could not send message: {:?}", why);
        }
        return;
    }
    say!(ctx, msg, "nothing matched your search");
}

fn send_element(ctx: &Context, msg: &Message, args: &[&str]) {
    if args.len() < 2 {
        say!(ctx, msg, "you must provide an element");
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

fn reload(ctx: &Context, msg: &Message, _ : &[&str]) {
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

fn roll(ctx: &Context, msg: &Message, args: &[&str]) {
    if args.len() < 2 {
        say!(ctx, msg, format!("{}, you must supply a number of dice to roll", msg.author.mention()));
    }

    //grab all but the first argument which is the command name
    let to_join = &args[1..];

    let to_roll = to_join.join(" ");
    let mut results : Vec<i64> = vec![];
    let amt = DieRoll::roll_dice(&to_roll, results.borrow_mut());
    let repl_str = format!("{:?}", results);
    let reply;
    if repl_str.len() > 1850 {
        reply = format!(
            "{}, you rolled: {}\n[There were too many die rolls to show the result of each one]",
            msg.author.mention(),
            amt
        );
    } else {
        reply = format!("{}, you rolled: {}\n{}", msg.author.mention(), amt, repl_str);
    }
    say!(ctx, msg, reply);

}

fn roll_stats(ctx: &Context, msg: &Message, _ : &[&str]) {
    let mut stats :[i64; 6] = [0;6];
    let mut rolls : Vec<i64> = vec![];
    for i in &mut stats {
        rolls.clear();
        DieRoll::roll_dice("4d6", &mut rolls);

        //sort reverse to put lowest at the end
        rolls.sort_unstable_by(|a,b| b.cmp(a));
        rolls.pop();

        *i = rolls.iter().fold(0, |acc, val| acc + val);
    }

    say!(ctx, msg, format!("{}, 4d6 drop the lowest:\n{:?}", msg.author.mention(), stats));
    /*
    if let Err(why) = msg.channel_id.say(
        &ctx.http,
    format!("{}, 4d6 drop the lowest:\n{:?}", msg.author.mention(), stats)
    ) {
        println!("Could not send message: {:?}", why);
    }
    */

}

fn send_ncp(ctx: &Context, msg: &Message, args: &[&str]) {
    if args.len() < 2 {
        say!(ctx, msg, "you must provide a name");
        return;
    }
    let data = ctx.data.read();
    let library_lock = data.get::<NCPLibrary>().expect("NCP library not found");
    let library = library_lock.read().expect("library was poisoned, panicking");
    search_lib_obj(ctx, msg, args[1], library.deref());
}

fn send_virus(ctx: &Context, msg: &Message, args: &[&str]) {
    if args.len() < 2 {
        say!(ctx, msg, "you must provide a name");
        return;
    }
    let to_join = &args[1..];
    let to_search = to_join.join(" ");
    let data = ctx.data.read();
    let library_lock = data.get::<VirusLibrary>().expect("NCP library not found");
    let library = library_lock.read().expect("library was poisoned, panicking");
    search_lib_obj(ctx, msg, &to_search, library.deref());
}

fn get_cr(ctx: &Context, msg: &Message, args: &[&str]) {
    if args.len() < 2 {
        say!(ctx, msg, "you must provide a CR to search for");
        return;
    }
    let cr_to_get_res = args[1].trim().parse::<u8>();

    if cr_to_get_res.is_err() {
        say!(ctx, msg, "an invalid number was provided");
        return;
    }
    let cr_to_get = cr_to_get_res.unwrap();
    let data = ctx.data.read();
    let library_lock = data.get::<VirusLibrary>().expect("NCP library not found");
    let library = library_lock.read().expect("library was poisoned, panicking");
    let response = library.get_cr(cr_to_get);
    say!(ctx, msg, response);
}

fn send_help(ctx: &Context, msg: &Message, _ : &[&str]) {
    say!(ctx, msg, format!("```{}```", *HELP));
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

