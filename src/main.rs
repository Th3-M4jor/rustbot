use std::{
    borrow::Cow,
    sync::Arc
};

use tokio::sync::RwLock;

use once_cell::sync::Lazy;

use serenity::{
    client::bridge::gateway::GatewayIntents,
    framework::standard::{
        help_commands,
        macros::{check, command, group, help, hook},
        Args, Reason, CommandGroup, CommandOptions, CommandResult, HelpOptions,
        StandardFramework,
    },
    model::{
        channel::Message,
        id::UserId,
    },
    prelude::*,
};

use crate::{
    bot_data::BotData,
    dice::DICE_GROUP,
    handler::Handler,
    library::{
        blights::{
            Blights, Panels, Statuses, GET_BLIGHT_COMMAND, GET_PANELS_COMMAND, GET_STATUS_COMMAND,
        },
        chip_library::{battlechip_as_lib_obj, ChipLibrary, BNBCHIPS_GROUP, BNBSKILLS_GROUP},
        full_library::{check_virus_drops, search_full_library, FullLibrary, CHIP_DROP_COMMAND},
        ncp_library::{ncp_as_lib_obj, NCPLibrary, BNBNCPS_GROUP},
        virus_library::{virus_as_lib_obj, VirusLibrary, BNBVIRUSES_GROUP},
        Library, LibraryObject,
    },
    util::{ShardManagerContainer, AUDIT_COMMAND, DIE_COMMAND, MANAGER_COMMAND, PHB_COMMAND, PING_COMMAND, SHUT_UP_COMMAND},
};

#[macro_use]
mod util;
mod bot_data;
mod dice;
mod library;
mod handler;

type ReloadOkType = (String, Vec<Arc<dyn LibraryObject>>);
type ReloadReturnType = Result<ReloadOkType, Box<dyn std::error::Error + Send + Sync>>;

static ABOUT_BOT: Lazy<String> = Lazy::new(|| 
    std::fs::read_to_string("./about.txt")
    .unwrap_or_else(|_| "about text is missing, bug the owner".to_string())
);

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
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[group]
#[owners_only]
#[help_available(false)]
#[commands(die, audit, shut_up)]
/// Administrative commands for the bot
struct Owner;

#[group]
#[commands(
    manager, phb, reload, get_blight, about_bot, chip_drop, get_status, get_panels, ping
)]
/// Misc. commands related to BnB
struct BnbGeneral;

#[check]
#[name = "Admin"]
async fn admin_check(
    ctx: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions,
) -> Result<(), Reason> {
    let data = ctx.data.read().await;
    let config = data.get::<BotData>().expect("could not get config");

    if msg.author.id == config.owner || config.admins.contains(&msg.author.id.0) {
        Ok(())
    } else {
        Err(Reason::User(format!("{} lacks permission for this command", &msg.author.name)))
    }
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

    let chip_future = ChipLibrary::reload(chip_data);
    let ncp_future = NCPLibrary::reload(ncp_data);
    let virus_future = VirusLibrary::reload(virus_data);

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
            reply!(
                ctx,
                msg,
                format!(
                    "An error occurred, library is not guaranteed to be in a usable state:\n{}",
                    e
                ),
                true
            );
            return Err(e);
        }
    }

    let data = ctx.data.read().await;
    let blight_string =
    {
        let blight_lock = data.get::<Blights>().expect("Blights not found");
        let mut blights = blight_lock.write().await;
        match blights.load().await {
            Ok(()) => Cow::Borrowed("blights reloaded successfully\n"),
            Err(e) => Cow::Owned(format!("{}\n", e.to_string())),
        }
    };

    let full_library_lock = data.get::<FullLibrary>().expect("full library not found");
    let mut full_library = full_library_lock.write().await;
    let mut full_duplicates: Vec<String> = Vec::new();
    full_library.clear();

    let full_iter = chip_res.1.iter().chain(ncp_res.1.iter()).chain(virus_res.1.iter());

    for lib_obj in full_iter {
        if let Err(e) = full_library.insert(Arc::clone(lib_obj)) {
            full_duplicates.push(e.to_string());
        }
    }

    let mut str_to_send = if full_duplicates.len() > 0 {
        format!("{}{}{}\n{}\nfull duplicates: {:?}", chip_res.0, ncp_res.0, virus_res.0, blight_string, full_duplicates)
    } else {
        format!("{}{}{}\n{}", chip_res.0, ncp_res.0, virus_res.0, blight_string)
    };

    let virus_lib_lock = data.get::<VirusLibrary>().expect("virus library not found");
    let chip_lib_lock = data.get::<ChipLibrary>().expect("chip library not found");
    let chip_lib = chip_lib_lock.read().await;
    let virus_lib = virus_lib_lock.read().await;

    if let Err(why) = check_virus_drops(&virus_lib, &chip_lib) {
        str_to_send.push_str(&why.to_string());
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
    reply!(
        ctx,
        msg,
        "You gave me only my prefix, Try my help command for how I work"
    );
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let config = BotData::new();

    // create futures for importing panels, blights, and statuses
    let blights_future = Blights::import();
    let statuses_future = Statuses::import();
    let panels_future = Panels::import();

    // join all three futures at once, panic if any one returns an error, unrecoverable
    let (blights, statuses, panels) =
        tokio::try_join!(blights_future, statuses_future, panels_future)
            .expect("import of blights, statuses, or panels failed. Aborting.");

    // create library structs
    let mut chip_library = ChipLibrary::new(&config.chip_url, &config.custom_chip_url);
    let mut ncp_library = NCPLibrary::new(&config.ncp_url);
    let mut virus_library = VirusLibrary::new(&config.virus_url);
    let mut full_library = FullLibrary::new();

    // create library struct import futures
    let chip_load_fut = chip_library.load_chips(config.load_custom_chips);
    let ncp_load_fut = ncp_library.load_programs();
    let virus_load_fut = virus_library.load_viruses();

    // join all futures at once
    let (chip_res, ncp_res, virus_res) = tokio::join!(chip_load_fut, ncp_load_fut, virus_load_fut);

    // panic if chip or ncp librarys are unusable
    let chip_count = chip_res.unwrap();
    println!("{} chips were loaded", chip_count);
    let ncp_count = ncp_res.unwrap();
    println!("{} programs loaded", ncp_count);

    // some virus import errors are recoverable, panic of they aren't
    match virus_res {
        Ok(s) => println!("{}", s),
        Err(e) => {
            println!("{}", e);
            if e.is_unrecoverable() {
                panic!();
            }
        }
    }

    // load chips, viruses, ncps into the full library struct, panic on unrecoverable duplicates
    for val in chip_library.get_collection().values() {
        let obj = battlechip_as_lib_obj(Arc::clone(val));
        if let Err(e) = full_library.insert(obj) {
            println!("Found duplicate name in full library: {}", e.as_str());
            panic!();
        }
    }

    for val in virus_library.get_collection().values() {
        let obj = virus_as_lib_obj(Arc::clone(val));
        if let Err(e) = full_library.insert(obj) {
            println!("Found duplicate name in full library: {}", e.as_str());
            panic!();
        }
    }

    for val in ncp_library.get_collection().values() {
        let obj = ncp_as_lib_obj(Arc::clone(val));
        if let Err(e) = full_library.insert(obj) {
            println!("Found duplicate name in full library: {}", e.as_str());
            panic!();
        }
    }

    println!("Full library loaded, size is {}", full_library.len());

    // configure standard framework
    let mut owners = std::collections::HashSet::new();
    let owner_id = serenity::model::id::UserId(config.owner);
    owners.insert(owner_id);
    let prefix = config.cmd_prefix.clone();
    let framework = StandardFramework::new()
        .configure(move |c| {
            c.with_whitespace(true)
                .prefix(&prefix)
                .case_insensitivity(true)
                .owners(owners)
                .delimiters(vec![" ", ", "])
        })
        //.unrecognised_command(search_everything_command)
        .normal_message(default_message)
        .prefix_only(prefix_only_message)
        .bucket("Warframe_Market", |b| b.delay(5))
        .await
        .help(&HELP_COMMAND)
        .group(&OWNER_GROUP)
        .group(&DICE_GROUP)
        .group(&BNBGENERAL_GROUP)
        .group(&BNBCHIPS_GROUP)
        .group(&BNBSKILLS_GROUP)
        .group(&BNBVIRUSES_GROUP)
        .group(&BNBNCPS_GROUP);

    let mut client = Client::builder(&config.token)
        .event_handler(Handler)
        .framework(framework)
        .intents(
            GatewayIntents::GUILD_MESSAGE_REACTIONS
                | GatewayIntents::DIRECT_MESSAGES
                | GatewayIntents::GUILD_MESSAGES
                | GatewayIntents::GUILDS,
        )
        .await
        .expect("Err creating client");

    // insert stuff into the data Arc<RwLock>
    {
        let mut data = client.data.write().await;
        data.insert::<ChipLibrary>(RwLock::new(chip_library));
        data.insert::<NCPLibrary>(RwLock::new(ncp_library));
        data.insert::<VirusLibrary>(RwLock::new(virus_library));
        data.insert::<BotData>(config);
        data.insert::<FullLibrary>(RwLock::new(full_library));
        data.insert::<Blights>(blights);
        data.insert::<Statuses>(statuses);
        data.insert::<Panels>(panels);
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
