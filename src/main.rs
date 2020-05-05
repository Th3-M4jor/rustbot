use lazy_static::lazy_static;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::{RwLock, RwLockWriteGuard};

use serenity::{
    async_trait,
    client::bridge::gateway::GatewayIntents,
    framework::standard::{
        help_commands, macros::*, Args, CheckResult, CommandError, CommandGroup, CommandOptions,
        CommandResult, HelpOptions, StandardFramework,
    },
    model::{
        channel::Message,
        event::ResumedEvent,
        gateway::{Activity, Ready},
        id::{GuildId, UserId},
    },
    prelude::*,
    utils::TypeMap,
};

use crate::{
    bot_data::BotData,
    library::{
        blights::*, chip_library::*, full_library::*, ncp_library::*, virus_library::*, Library,
        LibraryObject,
    },
    warframe::*,
};

use crate::{dice::*, util::*};
use std::fs;

// use regex::Replacer;
#[macro_use]
mod util;
mod bot_data;
mod dice;
mod library;
mod warframe;

// type BotCommand = fn(Context, &Message, &[&str]) -> ();

type ReloadOkType = (String, Vec<Arc<dyn LibraryObject>>);
type ReloadReturnType = Result<ReloadOkType, Box<dyn std::error::Error + Send + Sync>>;

lazy_static! {
    static ref ABOUT_BOT: String = fs::read_to_string("./about.txt")
        .unwrap_or_else(|_| "about text is missing, bug the owner".to_string());
}

struct Handler;

struct DmOwner;

impl TypeMapKey for DmOwner {
    type Value = AtomicBool;
}

lazy_static! {
    static ref FIRST_LOGIN: AtomicBool = AtomicBool::new(true);
    static ref FIRST_CACHE_READY: AtomicBool = AtomicBool::new(false);
}

#[async_trait]
impl EventHandler for Handler {
    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        if FIRST_CACHE_READY.compare_and_swap(false, true, Ordering::AcqRel) {
            // previous value was already true, return
            return;
        }

        println!("{} : Cache Ready", chrono::Local::now());

        {
            let data = ctx.data.read().await;
            let config = data.get::<BotData>().expect("no bot data, panicking");

            let action = config.cmd_prefix.clone() + "help for a list of commands";
            ctx.set_activity(Activity::playing(&action)).await;
        }

        if let Err(why) = dm_owner(&ctx, "logged in, and cache ready").await {
            println!("{:?}", why);
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        let message_to_owner;
        if FIRST_LOGIN.compare_and_swap(true, false, Ordering::AcqRel) {
            println!(
                "{} : {} is connected!",
                chrono::Local::now(),
                ready.user.name
            );
            return;
        } else {
            message_to_owner = "ready event re-emitted";
            println!(
                "{} : ready event re-emitted:\n{:?}",
                chrono::Local::now(),
                ready.trace
            );
        }

        // tokio::time::delay_for(std::time::Duration::from_secs(3)).await;

        {
            let data = ctx.data.read().await;
            let config = data.get::<BotData>().expect("no bot data, panicking");

            let action = config.cmd_prefix.clone() + "help for a list of commands";
            ctx.set_activity(Activity::playing(&action)).await;
        }

        if let Err(why) = dm_owner(&ctx, message_to_owner).await {
            println!("{:?}", why);
        }
    }

    async fn resume(&self, ctx: Context, resumed: ResumedEvent) {
        // let owner = fetch_owner(&ctx).await.expect("Could not fetch owner");
        // let owner_user = owner.user.read().await;
        let message_to_owner = "resume event was emitted";

        if let Err(why) = dm_owner(&ctx, message_to_owner).await {
            println!("{:?}", why);
        }

        println!(
            "{} : resume event emitted:\n{:?}",
            chrono::Local::now(),
            resumed.trace
        );
    }
}

async fn dm_owner<T>(
    ctx: &Context,
    to_send: T,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    T: std::fmt::Display,
{
    let data = ctx.data.read().await;

    let should_dm_owner = data.get::<DmOwner>().expect("No DM Owner setting found");

    if !should_dm_owner.load(Ordering::Relaxed) {
        return Ok(());
    }

    let config = data.get::<BotData>().expect("no bot data, panicking");

    let owner_id = UserId::from(config.owner);

    if let Some(owner_lock) = ctx.cache.read().await.users.get(&owner_id) {
        let owner = owner_lock.read().await;
        let _ = owner.dm(ctx, |m| m.content(format!("{}", to_send))).await?;
    } else {
        let owner = ctx.http.get_user(config.owner).await?;
        let _ = owner.dm(ctx, |m| m.content(format!("{}", to_send))).await?;
    }
    Ok(())
}

#[help]
#[max_levenshtein_distance(3)]
#[lacking_permissions = "hide"]
#[lacking_ownership = "hide"]
#[command_not_found_text = "Could not find: `{}`."]
#[strikethrough_commands_tip_in_dm(" ")]
#[strikethrough_commands_tip_in_guild(" ")]
#[individual_command_tip = "If you want more information about a specific command, just pass the \
                            command as an argument.\nIf an unknown command name is given, all \
                            Battlechips, Navi-Customizer Parts, and Viruses are searched for that \
                            name with battlechips being prioritized. (NCP's and Viruses may have a \
                            _n or _v appended to their name if there is a chip with that name)"]
async fn help_command(
    context: &Context,
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
#[help_available(false)]
#[commands(die, audit, shut_up)]
/// Administrative commands for the bot
struct Owner;

#[group]
#[commands(manager, phb, reload, get_blight, about_bot, chip_drop)]
/// Misc. commands related to BnB
struct BnbGeneral;

async fn reload_chips(data: Arc<RwLock<TypeMap>>) -> ReloadReturnType {
    let str_to_ret;
    let mut vec_to_ret: Vec<Arc<dyn LibraryObject>> = vec![];
    let data_lock = data.read().await;
    let chip_library_lock = data_lock
        .get::<ChipLibrary>()
        .expect("chip library not found");
    let mut chip_library: RwLockWriteGuard<ChipLibrary> = chip_library_lock.write().await;
    let chip_reload_str = chip_library.load_chips().await?;
    str_to_ret = format!("{} chips loaded\n", chip_reload_str);
    vec_to_ret.reserve(chip_library.get_collection().len());
    for val in chip_library.get_collection().values() {
        let trait_obj = battlechip_as_lib_obj(Arc::clone(val));

        vec_to_ret.push(trait_obj);
    }
    Ok((str_to_ret, vec_to_ret))
}

async fn reload_ncps(data: Arc<RwLock<TypeMap>>) -> ReloadReturnType {
    let str_to_ret: String;
    let mut vec_to_ret: Vec<Arc<dyn LibraryObject>> = vec![];
    let data_lock = data.read().await;
    let ncp_library_lock = data_lock
        .get::<NCPLibrary>()
        .expect("ncp library not found");
    let mut ncp_library = ncp_library_lock.write().await;
    let count = ncp_library.load_programs().await?;
    str_to_ret = format!("{} NCPs loaded\n", count);
    vec_to_ret.reserve(count);
    for val in ncp_library.get_collection().values() {
        vec_to_ret.push(ncp_as_lib_obj(Arc::clone(val)));
    }
    Ok((str_to_ret, vec_to_ret))
}

async fn reload_viruses(data: Arc<RwLock<TypeMap>>) -> ReloadReturnType {
    let mut vec_to_ret: Vec<Arc<dyn LibraryObject>> = vec![];
    let data_lock = data.read().await;
    let virus_library_lock = data_lock
        .get::<VirusLibrary>()
        .expect("virus library not found");
    let mut virus_library: RwLockWriteGuard<VirusLibrary> = virus_library_lock.write().await;
    let str_to_ret = virus_library.load_viruses().await?;

    vec_to_ret.reserve(virus_library.get_collection().len());
    for val in virus_library.get_collection().values() {
        vec_to_ret.push(virus_as_lib_obj(Arc::clone(val)));
    }

    Ok((str_to_ret, vec_to_ret))
}

#[check]
#[name = "Admin"]
async fn admin_check(
    ctx: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions,
) -> CheckResult {
    let data = ctx.data.read().await;
    let config = data.get::<BotData>().expect("could not get config");

    return (msg.author.id == config.owner || config.admins.contains(msg.author.id.as_u64()))
        .into();
}

#[command]
#[checks(Admin)]
/// Reload all Blights, BattleChips, NaviCust Parts, and Viruses
async fn reload(ctx: &Context, msg: &Message, _: Args) -> CommandResult {
    println!(
        "{} : Reload command called by: {}",
        chrono::Local::now(),
        msg.author.name
    );

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
    let data = ctx.data.read().await;
    let blight_string;
    {
        let blight_lock = data.get::<Blights>().expect("Blights not found");
        let mut blights = blight_lock.write().await;
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
/// Get some more information about the bot itself
async fn about_bot(ctx: &Context, msg: &Message, _: Args) -> CommandResult {
    let res = msg
        .author
        .dm(ctx, |m| m.content(format!("```{}```", *ABOUT_BOT)))
        .await;
    if res.is_err() {
        println!("Could not send about message: {:?}", res.unwrap_err());
    }
    Ok(())
}

#[command("shut_up")]
/// Makes the bot stop DMing the owner on certain events
async fn shut_up(ctx: &Context, msg: &Message, _: Args) -> CommandResult {
    {
        let data = ctx.data.read().await;

        // fetch and xor means fewer operations whole true ^ true is false, and true ^ false is true;
        let _res = data
            .get::<DmOwner>()
            .expect("No DM Owner setting found")
            .fetch_xor(true, Ordering::Relaxed);

        #[cfg(debug_assertions)]
        println!("DMing owner set to: {}", !_res);
    }
    msg.react(ctx, '\u{1f44d}').await?;
    return Ok(());
}

// #[hook]
// async fn search_everything_command(ctx: &mut Context, msg: &Message, _: &str) {
// let mut args: Vec<&str>;
// let new_first;
//
// {
// let data = ctx.data.read().await;
// let config = data.get::<BotData>().expect("no config found");
// if !msg.content.starts_with(&config.cmd_prefix) {
// return;
// }
// #[cfg(debug_assertions)]
// println!("unrecognized command called");
// args = msg.content.split(" ").collect();
// new_first = args[0].replacen(&config.cmd_prefix, "", 1);
// args[0] = new_first.as_str();
// }
//
// search_full_library(ctx, msg, &args).await;
// }

#[hook]
async fn default_message(ctx: &Context, msg: &Message) {
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
        args = msg.content.split(' ').collect();
        new_first = args[0].replacen(&config.cmd_prefix, "", 1);
        args[0] = new_first.as_str();
    }

    search_full_library(ctx, msg, &args).await;
}

#[hook]
async fn prefix_only_message(ctx: &Context, msg: &Message) {
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
    pretty_env_logger::init();
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
        
        // println!("{} programs loaded", ncp_count);
        let mut virus_library = virus_library_mutex.write().await;
        match virus_library.load_viruses().await {
            Ok(s) => println!("{}", s),
            Err(e) => println!("{}", e.to_string()),
        }

        let mut full_library = full_library_mutex.write().await;
        for val in chip_library.get_collection().values() {
            let obj = battlechip_as_lib_obj(Arc::clone(val));
            if let Err(e) = full_library.insert(obj) {
                println!("Found duplicate name in full library: {}", e.as_str());
            }
        }
        for val in virus_library.get_collection().values() {
            let obj = virus_as_lib_obj(Arc::clone(val));
            if let Err(e) = full_library.insert(obj) {
                println!("Found duplicate name in full library: {}", e.as_str());
            }
        }
        for val in ncp_library.get_collection().values() {
            let obj = ncp_as_lib_obj(Arc::clone(val));
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

    // let mut client = Client::new_with_framework(&config.token, Handler, framework)
    // .await
    // .expect("Err creating client");
    let mut client = Client::new(&config.token).event_handler(Handler).framework(framework).intents(
        GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILDS,
    ).await.expect("Err creating client");
    
    /*
    let mut client = Client::new_with_extras(&config.token, move |f| {
        f.framework(framework)
            .intents(
                GatewayIntents::GUILD_MESSAGE_REACTIONS
                    | GatewayIntents::DIRECT_MESSAGES
                    | GatewayIntents::GUILD_MESSAGES
                    | GatewayIntents::GUILDS,
            )
            .event_handler(Handler)
    })
    .await
    .expect("Err creating client");
    */
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
        data.insert::<DmOwner>(AtomicBool::new(true));
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
