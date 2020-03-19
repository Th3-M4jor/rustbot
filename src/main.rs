#[macro_use]
extern crate lazy_static;

use std::process::exit;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

use serenity::{
    async_trait,
    framework::standard::{
        macros::{command, group, hook},
        Args, CommandResult, StandardFramework,
    },
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

use crate::bot_data::BotData;
use crate::library::{
    blights::*, chip_library::*, full_library::*, ncp_library::*, virus_library::*, Library,
};
use crate::warframe::{market::*, *};

use crate::dice::*;
use crate::util::*;
use serenity::model::gateway::Activity;
use std::fs;

//use regex::Replacer;
#[macro_use]
mod util;
mod bot_data;
mod dice;
mod library;
mod warframe;

//type BotCommand = fn(Context, &Message, &[&str]) -> ();

lazy_static! {

    /*
    static ref COMMANDS: HashMap<String, BotCommand> = {
        let mut cmd_map = HashMap::new();

        //need cast to BotCommand here once and rest are implicit to avoid compiler error due to type mismatch
        cmd_map.insert("chip".to_string(), send_chip as BotCommand);

        cmd_map.insert("cr".to_string(), send_virus_cr);
        cmd_map.insert("element".to_string(), send_chip_element);

        cmd_map.insert("ncp".to_string(), send_ncp);
        cmd_map.insert("ncpcolor".to_string(), send_ncp_color);
        cmd_map.insert("blight".to_string(), get_blight);

        cmd_map.insert("reload".to_string(), reload);
        cmd_map.insert("die".to_string(), check_exit);

        cmd_map.insert("skill".to_string(), send_chip_skill);
        cmd_map.insert("skilluser".to_string(), send_chip_skill);
        cmd_map.insert("skilltarget".to_string(), send_chip_skill);
        cmd_map.insert("skillcheck".to_string(), send_chip_skill);

        cmd_map.insert("roll".to_string(), roll);
        cmd_map.insert("reroll".to_string(), roll);
        cmd_map.insert("rollstats".to_string(), roll_stats);

        cmd_map.insert("virus".to_string(), send_virus);
        cmd_map.insert("encounter".to_string(), send_random_encounter);
        cmd_map.insert("viruselement".to_string(), send_virus_element);
        cmd_map.insert("family".to_string(), send_family);

        cmd_map.insert("help".to_string(), send_help);
        cmd_map.insert("about".to_string(), about_bot);
        cmd_map.insert("audit".to_string(), audit_log);
        cmd_map.insert("manager".to_string(), manager);
        cmd_map.insert("phb".to_string(), send_handbook);

        cmd_map.insert("sortie".to_string(), get_sortie);
        cmd_map.insert("fissures".to_string(), get_fissures);
        cmd_map.insert("market".to_string(), get_market_info);
        return cmd_map;
    };
    */

    static ref HELP_STRING: String = fs::read_to_string("./help.txt").unwrap_or("help text is missing, bug the owner".to_string());

    static ref ABOUT_BOT: String = fs::read_to_string("./about.txt").unwrap_or("about text is missing, bug the owner".to_string());
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    /*
    fn message(&self, ctx: Context, msg: Message) {
        //let msg_content_clone;
        let mut args: Vec<&str>;
        let new_first;

        {
            let data = ctx.data.read();
            let config = data.get::<BotData>().expect("no config found");
            if !msg.content.starts_with(&config.cmd_prefix) {
                return;
            }
            //msg_content_clone = msg.content.clone();
            args = msg.content.split(" ").collect();
            new_first = args[0].replacen(&config.cmd_prefix, "", 1);
            args[0] = new_first.as_str();
        }

        //get the command from a jump table
        let cmd_res = COMMANDS.get(&args[0].to_lowercase());
        match cmd_res {
            Some(cmd) => cmd(ctx, &msg, &args),
            None => search_full_library(ctx, &msg, &args),
        }
    }
    */

    async fn ready(&self, ctx: Context, ready: Ready) {
        lazy_static! {
            static ref FIRST_LOGIN: AtomicBool = AtomicBool::new(true);
        }
        let data = ctx.data.read().await;
        let config = data.get::<BotData>().expect("no bot data, panicking");

        let guild = serenity::model::guild::Guild::get(&ctx, config.main_server)
            .await
            .expect("could not find main server");
        let owner = guild
            .member(&ctx, config.owner)
            .await
            .expect("could not grab owner");
        let message_to_owner;
        if FIRST_LOGIN.load(Ordering::Relaxed) {
            message_to_owner = "logged in, and ready";
            println!("{} is connected!", ready.user.name);
            FIRST_LOGIN.store(false, Ordering::Relaxed);
        } else {
            message_to_owner = "an error occurred, reconnected and ready";
            println!("{:?}", ready.trace);
        }
        let owner_user = owner.user.read().await;
        owner_user
            .dm(&ctx, |m| {
                m.content(message_to_owner);
                return m;
            })
            .await
            .expect("could not dm owner");
        let action = config.cmd_prefix.clone() + "help for a list of commands";
        ctx.set_activity(Activity::playing(&action)).await;
    }
}

#[group]
#[commands(
    manager, phb, die, reload, audit, send_help, get_blight, roll, roll_stats
)]
struct General;

#[group]
#[commands(get_sortie, get_fissures, market)]
struct Warframe;

#[group]
#[commands(send_chip, send_chip_skill, send_chip_element)]
struct BnbChips;

#[group]
#[commands(
    send_virus,
    send_virus_element,
    send_virus_cr,
    send_random_encounter,
    send_family
)]
struct BnbViruses;

#[group]
#[commands(send_ncp, send_ncp_color)]
struct BnbNcps;

#[command]
async fn manager(ctx: &mut Context, msg: &Message, _: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let config = data.get::<BotData>().expect("could not get config");
    say!(ctx, msg, &config.manager);
    return Ok(());
}

#[command]
async fn phb(ctx: &mut Context, msg: &Message, _: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let config = data.get::<BotData>().expect("could not get config");
    say!(ctx, msg, &config.phb);
    return Ok(());
}

#[command]
async fn die(ctx: &mut Context, msg: &Message, _: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let config = data.get::<BotData>().expect("config not found");

    if msg.author.id == config.owner {
        ctx.invisible().await;
        exit(0);
    }
    return Ok(());
}

async fn reload_chips(data: Arc<RwLock<ShareMap>>) -> (String, Vec<FullLibraryType>) {
    let str_to_ret;
    let mut vec_to_ret: Vec<FullLibraryType> = vec![];
    let data_lock = data.read().await;
    let chip_library_lock = data_lock
        .get::<ChipLibrary>()
        .expect("chip library not found");
    let mut chip_library = chip_library_lock.write().await;
    let chip_reload_res = chip_library.load_chips();
    //let str_to_send;
    match chip_reload_res.await {
        Ok(s) => str_to_ret = format!("{} chips loaded\n", s),
        Err(e) => str_to_ret = format!("{}\n", e.to_string()),
    }
    vec_to_ret.reserve(chip_library.get_collection().len());
    for val in chip_library.get_collection().values() {
        vec_to_ret.push(FullLibraryType::BattleChip(Arc::clone(val)));
    }
    return (str_to_ret, vec_to_ret);
}

async fn reload_ncps(data: Arc<RwLock<ShareMap>>) -> (String, Vec<FullLibraryType>) {
    let str_to_ret: String;
    let mut vec_to_ret: Vec<FullLibraryType> = vec![];
    let data_lock = data.read().await;
    let ncp_library_lock = data_lock
        .get::<NCPLibrary>()
        .expect("ncp library not found");
    let mut ncp_library = ncp_library_lock.write().await;
    let count = ncp_library.load_programs().await;
    str_to_ret = format!("{} NCPs loaded\n", count);
    vec_to_ret.reserve(count);
    for val in ncp_library.get_collection().values() {
        vec_to_ret.push(FullLibraryType::NCP(Arc::clone(val)));
    }
    return (str_to_ret, vec_to_ret);
}

async fn reload_viruses(data: Arc<RwLock<ShareMap>>) -> (String, Vec<FullLibraryType>) {
    let str_to_ret: String;
    let mut vec_to_ret: Vec<FullLibraryType> = vec![];
    let data_lock = data.read().await;
    let virus_library_lock = data_lock
        .get::<VirusLibrary>()
        .expect("virus library not found");
    let mut virus_library = virus_library_lock.write().await;
    //.expect("virus library was poisoned, panicking");
    match virus_library.load_viruses().await {
        Ok(s) => str_to_ret = format!("{} viruses were loaded\n", s),
        Err(e) => str_to_ret = format!("{}", e.to_string()),
    }

    vec_to_ret.reserve(virus_library.get_collection().len());
    for val in virus_library.get_collection().values() {
        vec_to_ret.push(FullLibraryType::Virus(Arc::clone(val)));
    }

    return (str_to_ret, vec_to_ret);
}

#[command]
async fn reload(ctx: &mut Context, msg: &Message, _: Args) -> CommandResult {
    {
        let data = ctx.data.read().await;
        let config = data.get::<BotData>().expect("could not get config");
        if msg.author.id != config.owner && !config.admins.contains(msg.author.id.as_u64()) {
            return Ok(());
        }
    }

    if let Err(_) = msg.channel_id.broadcast_typing(&ctx.http).await {
        println!("could not broadcast typing, not reloading");
        return Ok(());
    }

    let chip_data = Arc::clone(&ctx.data);
    let ncp_data = Arc::clone(&ctx.data);
    let virus_data = Arc::clone(&ctx.data);

    let mut str_to_send = String::new();
    let chip_future = reload_chips(chip_data);
    let ncp_future = reload_ncps(ncp_data);
    let virus_future = reload_viruses(virus_data);
    let (chip_res, ncp_res, virus_res) = tokio::join!(chip_future, ncp_future, virus_future);

    let data = ctx.data.read().await;
    let blight_string;
    {
        let blight_lock = data.get::<Blights>().expect("Blights not found");
        let mut blights = blight_lock.write().await; //.expect("blights poisoned, panicking");
        match blights.load().await {
            Ok(()) => blight_string = String::from("blights reloaded successfully\n"),
            Err(e) => blight_string = format!("{}\n", e.to_string()),
        }
    }

    let full_library_lock = data.get::<FullLibrary>().expect("full library not found");
    let mut full_library = full_library_lock.write().await;
    let mut full_duplicates: Vec<String> = vec![];
    full_library.clear();
    for chip in chip_res.1 {
        if let Err(e) = full_library.insert(chip) {
            full_duplicates.push(e.to_string());
        }
    }

    for ncp in ncp_res.1 {
        if let Err(e) = full_library.insert(ncp) {
            full_duplicates.push(e.to_string());
        }
    }

    for virus in virus_res.1 {
        if let Err(e) = full_library.insert(virus) {
            full_duplicates.push(e.to_string());
        }
    }

    str_to_send.push_str(&chip_res.0);
    str_to_send.push_str(&ncp_res.0);
    str_to_send.push_str(&virus_res.0);
    str_to_send.push_str(&blight_string);

    if full_duplicates.len() > 0 {
        str_to_send.push_str(&format!("\nfull duplicates: {:?}", full_duplicates));
    }

    say!(ctx, msg, str_to_send);
    return Ok(());
}

#[command]
#[aliases("help")]
async fn send_help(ctx: &mut Context, msg: &Message, _: Args) -> CommandResult {
    let res = msg
        .author
        .dm(ctx, |m| {
            m.content(format!("```{}```", *HELP_STRING));
            return m;
        })
        .await;
    if res.is_err() {
        println!("Could not send help message: {:?}", res.unwrap_err());
    }
    return Ok(());
}

#[command]
#[aliases("about")]
async fn about_bot(ctx: &mut Context, msg: &Message, _: Args) -> CommandResult {
    let res = msg
        .author
        .dm(ctx, |m| {
            m.content(format!("```{}```", *ABOUT_BOT));
            return m;
        })
        .await;
    if res.is_err() {
        println!("Could not send help message: {:?}", res.unwrap_err());
    }
    return Ok(());
}

#[hook]
async fn default_command(ctx: &mut Context, msg: &Message, _: &str) {
    println!("I have recieved an unrecognized command");
    let mut args: Vec<&str>;
    let new_first;

    {
        let data = ctx.data.read().await;
        let config = data.get::<BotData>().expect("no config found");
        if !msg.content.starts_with(&config.cmd_prefix) {
            return;
        }
        //msg_content_clone = msg.content.clone();
        args = msg.content.split(" ").collect();
        new_first = args[0].replacen(&config.cmd_prefix, "", 1);
        args[0] = new_first.as_str();
    }

    search_full_library(ctx, msg, &args).await;
}

#[tokio::main]
async fn main() {
    let chip_library_mutex = RwLock::new(ChipLibrary::new());
    let ncp_library_mutex = RwLock::new(NCPLibrary::new());
    let virus_library_mutex = RwLock::new(VirusLibrary::new());
    let warframe_data = WarframeData::new();
    let full_library_mutex = RwLock::new(FullLibrary::new());
    let blight_mutex = RwLock::new(Blights::new());
    //let mut chip_library = ChipLibrary::new();

    {
        let mut blights = blight_mutex.write().await;
        match blights.load().await {
            Ok(()) => {
                println!("blights loaded");
            }
            Err(e) => {
                println!("{}", e.to_string());
            }
        }
        let mut chip_library = chip_library_mutex.write().await;
        match chip_library.load_chips().await {
            Ok(s) => {
                println!("{} chips were loaded", s);
            }
            Err(e) => {
                println!("{}", e.to_string());
            }
        }
        let mut ncp_library = ncp_library_mutex.write().await;
        let ncp_count = ncp_library.load_programs().await;
        println!("{} programs loaded", ncp_count);
        let mut virus_library = virus_library_mutex.write().await;
        match virus_library.load_viruses().await {
            Ok(s) => println!("{} viruses were loaded", s),
            Err(e) => println!("{}", e.to_string()),
        }

        let mut full_library = full_library_mutex.write().await;
        for val in chip_library.get_collection().values() {
            let obj = FullLibraryType::BattleChip(Arc::clone(val));
            if let Err(e) = full_library.insert(obj) {
                println!("Found duplicate name in full library: {}", e.as_str());
            }
        }
        for val in virus_library.get_collection().values() {
            let obj = FullLibraryType::Virus(Arc::clone(val));
            if let Err(e) = full_library.insert(obj) {
                println!("Found duplicate name in full library: {}", e.as_str());
            }
        }
        for val in ncp_library.get_collection().values() {
            let obj = FullLibraryType::NCP(Arc::clone(val));
            if let Err(e) = full_library.insert(obj) {
                println!("Found duplicate name in full library: {}", e.as_str());
            }
        }
        println!("Full library loaded, size is {}", full_library.len());
    }

    let config = BotData::new();
    let prefix = config.cmd_prefix.clone();
    let framework = StandardFramework::new()
        .configure(move |c| {
            c.with_whitespace(true)
                .prefix(&prefix)
                .case_insensitivity(true)
        })
        .unrecognised_command(default_command)
        .group(&GENERAL_GROUP)
        .group(&BNBCHIPS_GROUP)
        .group(&BNBVIRUSES_GROUP)
        .group(&BNBNCPS_GROUP)
        .group(&WARFRAME_GROUP);
    let mut client = Client::new_with_framework(&config.token, Handler, framework)
        .await
        .expect("Err creating client");
    // set scope to ensure that lock is released immediately
    {
        let mut data = client.data.write().await;
        data.insert::<ChipLibrary>(chip_library_mutex);
        data.insert::<NCPLibrary>(ncp_library_mutex);
        data.insert::<VirusLibrary>(virus_library_mutex);
        data.insert::<BotData>(config);
        data.insert::<WarframeData>(warframe_data);
        data.insert::<FullLibrary>(full_library_mutex);
        data.insert::<Blights>(blight_mutex);
    }
    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
