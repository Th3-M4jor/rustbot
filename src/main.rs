#[macro_use]
extern crate lazy_static;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use serenity::{
    async_trait,
    framework::standard::{
        help_commands, macros::*, Args, CheckResult, CommandError, CommandGroup, CommandOptions,
        CommandResult, HelpOptions, StandardFramework,
    },
    model::{channel::Message, gateway::Activity, gateway::Ready, guild::PartialGuild, id::UserId},
    prelude::*,
};

use crate::bot_data::BotData;
use crate::library::{
    blights::*, chip_library::*, full_library::*, ncp_library::*, virus_library::*, Library,
};
use crate::warframe::*;

use crate::dice::*;
use crate::util::*;
use std::fs;

//use regex::Replacer;
#[macro_use]
mod util;
mod bot_data;
mod dice;
mod library;
mod warframe;

//type BotCommand = fn(Context, &Message, &[&str]) -> ();

type ReloadOkType = (String, Vec<FullLibraryType>);
type ReloadReturnType = Result<ReloadOkType, Box<dyn std::error::Error + Send + Sync>>;

lazy_static! {
    static ref ABOUT_BOT: String = fs::read_to_string("./about.txt")
        .unwrap_or("about text is missing, bug the owner".to_string());
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.

    async fn ready(&self, ctx: Context, ready: Ready) {
        lazy_static! {
            static ref FIRST_LOGIN: AtomicBool = AtomicBool::new(true);
        }
        let data: RwLockReadGuard<ShareMap> = ctx.data.read().await;
        let config = data.get::<BotData>().expect("no bot data, panicking");

        let guild: PartialGuild = serenity::model::guild::Guild::get(&ctx, config.main_server)
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

#[help]
#[max_levenshtein_distance(3)]
#[lacking_permissions = "hide"]
#[lacking_ownership = "hide"]
#[command_not_found_text = "Could not find: `{}`."]
#[strikethrough_commands_tip_in_dm(" ")]
#[strikethrough_commands_tip_in_guild(" ")]
#[individual_command_tip = "If you want more information about a specific command, just pass the command as an argument.\n\
If an unknown command name is given, all Battlechips, Navi-Customizer Parts, and Viruses are searched for that name with \
battlechips being prioritized. (NCP's and Viruses may have a _n or _v appended to their name if there is a chip with that name)"]
async fn help_command(
    context: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: std::collections::HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners).await
}

#[group]
#[owners_only]
#[commands(die, audit)]
#[description("Administrative commands for the bot")]
struct Owner;

#[group]
#[commands(manager, phb, reload, get_blight, about_bot)]
#[description("Misc. commands related to BnB")]
struct BnbGeneral;

async fn reload_chips(data: Arc<RwLock<ShareMap>>) -> ReloadReturnType {
    let str_to_ret;
    let mut vec_to_ret: Vec<FullLibraryType> = vec![];
    let data_lock = data.read().await;
    let chip_library_lock = data_lock
        .get::<ChipLibrary>()
        .expect("chip library not found");
    let mut chip_library: RwLockWriteGuard<ChipLibrary> = chip_library_lock.write().await;
    let chip_reload_str = chip_library.load_chips().await?;
    str_to_ret = format!("{} chips loaded\n", chip_reload_str);
    //let str_to_send;
    vec_to_ret.reserve(chip_library.get_collection().len());
    for val in chip_library.get_collection().values() {
        vec_to_ret.push(FullLibraryType::BattleChip(Arc::clone(val)));
    }
    return Ok((str_to_ret, vec_to_ret));
}

async fn reload_ncps(data: Arc<RwLock<ShareMap>>) -> ReloadReturnType {
    let str_to_ret: String;
    let mut vec_to_ret: Vec<FullLibraryType> = vec![];
    let data_lock = data.read().await;
    let ncp_library_lock = data_lock
        .get::<NCPLibrary>()
        .expect("ncp library not found");
    let mut ncp_library = ncp_library_lock.write().await;
    let count = ncp_library.load_programs().await?;
    str_to_ret = format!("{} NCPs loaded\n", count);
    vec_to_ret.reserve(count);
    for val in ncp_library.get_collection().values() {
        vec_to_ret.push(FullLibraryType::NCP(Arc::clone(val)));
    }
    return Ok((str_to_ret, vec_to_ret));
}

async fn reload_viruses(data: Arc<RwLock<ShareMap>>) -> ReloadReturnType {
    //let str_to_ret: String;
    let mut vec_to_ret: Vec<FullLibraryType> = vec![];
    let data_lock = data.read().await;
    let virus_library_lock = data_lock
        .get::<VirusLibrary>()
        .expect("virus library not found");
    let mut virus_library: RwLockWriteGuard<VirusLibrary> = virus_library_lock.write().await;
    //.expect("virus library was poisoned, panicking");
    let str_to_ret = virus_library.load_viruses().await?;

    vec_to_ret.reserve(virus_library.get_collection().len());
    for val in virus_library.get_collection().values() {
        vec_to_ret.push(FullLibraryType::Virus(Arc::clone(val)));
    }

    return Ok((str_to_ret, vec_to_ret));
}

#[check]
#[name = "Admin"]
async fn admin_check(
    ctx: &mut Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions,
) -> CheckResult {
    let data: RwLockReadGuard<ShareMap> = ctx.data.read().await;
    let config = data.get::<BotData>().expect("could not get config");

    return (msg.author.id == config.owner || config.admins.contains(msg.author.id.as_u64()))
        .into();
}

#[command]
#[checks(Admin)]
#[description("Reload all Blights, BattleChips, NaviCust Parts, and Viruses")]
async fn reload(ctx: &mut Context, msg: &Message, _: Args) -> CommandResult {
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

    let chip_res;
    let ncp_res;
    let virus_res;

    //let (chip_res, ncp_res, virus_res) = tokio::try_join!(chip_future, ncp_future, virus_future);

    let res: Result<
        (ReloadOkType, ReloadOkType, ReloadOkType),
        Box<dyn std::error::Error + Send + Sync>,
    > = tokio::try_join!(chip_future, ncp_future, virus_future);
    match res {
        Ok(val) => {
            chip_res = val.0;
            ncp_res = val.1;
            virus_res = val.2;
        }
        Err(e) => {
            say!(
                ctx,
                msg,
                format!(
                    "An error occurred, library is not guaranteed to be in a usable state:\n {}",
                    e.to_string()
                )
            );
            return Err(CommandError(e.to_string()));
        }
    }
    let data: RwLockReadGuard<ShareMap> = ctx.data.read().await;
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

#[command("about")]
#[description("get some information about the bot itself")]
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

/*
#[hook]
async fn search_everything_command(ctx: &mut Context, msg: &Message, _: &str) {
    let mut args: Vec<&str>;
    let new_first;

    {
        let data = ctx.data.read().await;
        let config = data.get::<BotData>().expect("no config found");
        if !msg.content.starts_with(&config.cmd_prefix) {
            return;
        }
        #[cfg(debug_assertions)]
        println!("unrecognized command called");
        args = msg.content.split(" ").collect();
        new_first = args[0].replacen(&config.cmd_prefix, "", 1);
        args[0] = new_first.as_str();
    }

    search_full_library(ctx, msg, &args).await;
}
*/

#[hook]
async fn default_message(ctx: &mut Context, msg: &Message) {
    let mut args: Vec<&str>;
    let new_first;
    {
        let data = ctx.data.read().await;
        let config = data.get::<BotData>().expect("no config found");
        if !msg.content.starts_with(&config.cmd_prefix) {
            return;
        }
        #[cfg(debug_assertions)]
        println!("Default message called");
        args = msg.content.split(" ").collect();
        new_first = args[0].replacen(&config.cmd_prefix, "", 1);
        args[0] = new_first.as_str();
    }

    search_full_library(ctx, msg, &args).await;
}

#[hook]
async fn prefix_only_message(ctx: &mut Context, msg: &Message) {
    #[cfg(debug_assertions)]
    println!("I recieved only a prefix");
    say!(
        ctx,
        msg,
        "You gave me only my prefix, Try my help command for how I work"
    );
}

#[tokio::main]
async fn main() {
    let config = BotData::new();
    let chip_library_mutex =
        RwLock::new(ChipLibrary::new(&config.chip_url, &config.custom_chip_url));
    let ncp_library_mutex = RwLock::new(NCPLibrary::new(&config.ncp_url));
    let virus_library_mutex = RwLock::new(VirusLibrary::new(&config.virus_url));
    let warframe_data = WarframeData::new();
    let full_library_mutex = RwLock::new(FullLibrary::new());
    let blight_mutex = RwLock::new(Blights::new());

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

        match ncp_library.load_programs().await {
            Ok(s) => {
                println!("{} programs loaded", s);
            }
            Err(e) => {
                println!("{}", e.to_string());
            }
        }

        //println!("{} programs loaded", ncp_count);
        let mut virus_library = virus_library_mutex.write().await;
        match virus_library.load_viruses().await {
            Ok(s) => println!("{}", s),
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
    let mut owners = std::collections::HashSet::new();
    let owner_id = serenity::model::id::UserId::from(config.owner);
    owners.insert(owner_id);
    let prefix = config.cmd_prefix.clone();
    let framework = StandardFramework::new()
        .configure(move |c| {
            c.with_whitespace(true)
                .prefix(&prefix)
                .case_insensitivity(true)
                .owners(owners)
        })
        //.unrecognised_command(search_everything_command)
        .normal_message(default_message)
        .prefix_only(prefix_only_message)
        .bucket("Warframe_Market", |b| b.delay(5))
        .await
        .help(&HELP_COMMAND)
        .group(&OWNER_GROUP)
        .group(&WARFRAME_GROUP)
        .group(&DICE_GROUP)
        .group(&BNBGENERAL_GROUP)
        .group(&BNBCHIPS_GROUP)
        .group(&BNBSKILLS_GROUP)
        .group(&BNBVIRUSES_GROUP)
        .group(&BNBNCPS_GROUP);

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
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }
    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    } else {
        println!("Bot shutdown successfully");
    }
}
