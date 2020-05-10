use serenity::{
    builder::EditMessage,
    client::bridge::gateway::ShardManager,
    framework::standard::{macros::command, Args, CommandResult},
    http::CacheHttp,
    model::{channel::{Message, ReactionType}, id::{ChannelId, UserId}, permissions::Permissions},
    prelude::*,
};

use std::{sync::Arc, time::Duration};

use crate::bot_data::BotData;
use tokio::fs;

/// fn say(ctx: Context, msg: Message, say: an expression returning a string)

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

macro_rules! say {
    ($ctx: ident, $msg: ident, $say: expr) => {
        if let Err(why) = $msg.channel_id.say(&$ctx.http, $say).await {
            println!("Could not send message: {:?}", why);
        }
    };
}

macro_rules! long_say {
    ($ctx: ident,  $msg: ident, $say: expr, $sep: expr) => {
        if let Err(why) = $crate::send_long_message(&$ctx, &$msg, $say, $sep).await {
            println!("Could not send message: {:?}", why);
        }
    };
}

pub(crate) async fn send_long_message<T, S>(
    ctx: &Context,
    msg: &Message,
    to_send: T,
    separator: S,
) -> serenity::Result<Message>
where
    T: std::iter::IntoIterator,
    T::Item: std::fmt::Display,
    S: Into<String>,
{
    let mut reply = String::new();
    let sep = separator.into();
    for val in to_send {
        let to_push = format!("{}", val);
        // a single message cannot be greater than 2000 chars
        if reply.len() + to_push.len() > 1950 {
            msg.channel_id.say(&ctx.http, &reply).await?;
            reply.clear();
        }
        reply.push_str(&to_push);
        reply.push_str(&sep);
    }
    // remove last seperator
    for _ in 0..sep.len() {
        reply.pop();
    }
    msg.channel_id.say(&ctx.http, &reply).await
}

pub(crate) fn build_time_rem(now: i64, end: i64) -> String {
    let time_rem = end - now;
    if time_rem < 0 {
        return String::from("Expired");
    }
    let hours_rem = time_rem / (60 * 60);
    let min_rem = (time_rem % (60 * 60)) / 60;
    let sec_rem = (time_rem % (60 * 60)) % 60;
    if hours_rem == 0 {
        format!("{:02}m:{:02}s", min_rem, sec_rem)
    } else {
        format!("{}h:{:02}m:{:02}s", hours_rem, min_rem, sec_rem)
    }
}

pub(crate) async fn edit_message_by_id<T: ToString>(
    cache_http: impl CacheHttp,
    channel_id: u64,
    message_id: u64,
    new_msg: T,
) -> Result<Message, serenity::Error> {
    let mut edited_text = EditMessage::default();
    edited_text.content(new_msg.to_string());
    let map = serenity::utils::hashmap_to_json_map(edited_text.0);
    let stringified_map = serde_json::Value::Object(map);
    cache_http
        .http()
        .edit_message(channel_id, message_id, &stringified_map)
        .await
}

/// Returns true if the bot has permission to manage messages and add reactions to the given channel
pub(crate) async fn has_reaction_perm(ctx: &Context, channel_id: ChannelId) -> bool {
    let channel_res = channel_id.to_channel(ctx).await;
    if channel_res.is_err() {
        return false;
    }

    let channel = channel_res.unwrap();
    let guild_channel;
    match channel.guild() {
        Some(chan) => guild_channel = chan,
        None => return false,
    }

    let current_user_id = ctx.cache.read().await.user.id;

    let permissions = guild_channel
        .permissions_for_user(ctx, current_user_id)
        .await
        .unwrap();

    permissions.contains(Permissions::ADD_REACTIONS | Permissions::MANAGE_MESSAGES)
}

const NUMBERS: &[&str] = &[
    "\u{31}\u{fe0f}\u{20e3}", // 1
    "\u{32}\u{fe0f}\u{20e3}", // 2
    "\u{33}\u{fe0f}\u{20e3}", // 3
    "\u{34}\u{fe0f}\u{20e3}", // 4
    "\u{35}\u{fe0f}\u{20e3}", // 5
    "\u{36}\u{fe0f}\u{20e3}", // 6
    "\u{37}\u{fe0f}\u{20e3}", // 7
    "\u{38}\u{fe0f}\u{20e3}", // 8
    "\u{39}\u{fe0f}\u{20e3}", // 9
];

/// Panics if len is 0 or greater than 9
pub(crate) async fn reaction_did_you_mean(ctx: &Context, msg: &Message, author_id: UserId, len: usize) -> Option<usize> {
    if len == 0 || len > 9 {
        panic!("Recieved invalid number for did you mean: {}", len);
    }
    if !has_reaction_perm(&ctx, msg.channel_id).await {
        return None;
    }
    let http_clone = Arc::clone(&ctx.http);
    let msg_id = msg.id.0;
    let channel_id = msg.channel_id.0;
    let all_reactions_added = tokio::spawn(async move {
        for number in NUMBERS.iter().take(len) {
            if let Err(why) = http_clone
                .create_reaction(
                    channel_id,
                    msg_id,
                    &ReactionType::Unicode((*number).to_string()),
                )
                .await
            {
                println!("Could not react to message: {:?}", why);
                return false;
            }
        }
        return true;
    });

    let mut reacted_number: Option<usize> = None;
    'outer: loop {
    if let Some(reaction) = msg.await_reaction(&ctx)
    .timeout(Duration::from_secs(30))
    .author_id(author_id)
    .await {
        let emoji = &reaction.as_inner_ref().emoji.as_data();
        let reacted_emoji = emoji.as_str();
        for num in NUMBERS.iter().take(len).zip(0..=len) {
            if *num.0 == reacted_emoji {
                reacted_number = Some(num.1);
                break 'outer;
            }
        }
    } else {
        break;
    }

    }


    if let Err(why) = all_reactions_added.await {
        println!("{:?}", why);
    }

    if let Err(why) = msg.delete_reactions(ctx).await {
        println!("Could not delete reactions: {:?}", why);
        return None;
    }

    reacted_number

}

#[command]
/// Get the last few lines of the server log file
pub(crate) async fn audit(ctx: &Context, msg: &Message, _: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let config = data.get::<BotData>().expect("config not found");

    if msg.author.id != config.owner {
        return Ok(());
    }

    let res = fs::read_to_string("./nohup.out").await;
    match res {
        Ok(val) => {
            let lines: Vec<&str> = val.split('\n').filter(|&i| !i.trim().is_empty()).collect();
            let len = lines.len() - 11;
            long_say!(ctx, msg, &lines[len..], "\n");
        }
        Err(err) => {
            say!(ctx, msg, format!("unable to get log, {}", err.to_string()));
        }
    }
    return Ok(());
}

#[command]
/// Get a link to the BnB Battlechip manager website
async fn manager(ctx: &Context, msg: &Message, _: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let config = data.get::<BotData>().expect("could not get config");
    say!(ctx, msg, &config.manager);
    return Ok(());
}

#[command]
/// Get a link to the BnB Players Handbook
async fn phb(ctx: &Context, msg: &Message, _: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let config = data.get::<BotData>().expect("could not get config");
    say!(ctx, msg, &config.phb);
    return Ok(());
}

#[command]
/// Tells the bot to "die" and it will try to shutdown gracefully
async fn die(ctx: &Context, msg: &Message, _: Args) -> CommandResult {
    let data = ctx.data.read().await;

    ctx.invisible().await;
    if let Some(manager) = data.get::<ShardManagerContainer>() {
        manager.lock().await.shutdown_all().await;
    } else {
        let _ = msg.reply(ctx, "There was a problem getting the shard manager");
        std::process::exit(1);
    }
    return Ok(());
}
