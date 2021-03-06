use serenity::{
    client::bridge::gateway::ShardManager,
    framework::standard::{macros::command, Args, CommandResult},
    http::{CacheHttp, Http},
    model::{
        channel::{Message, ReactionType},
        id::{ChannelId, MessageId, UserId},
        permissions::Permissions,
    },
    prelude::*,
};

use std::{sync::{Arc, atomic::{AtomicBool, Ordering}}, time::Duration};

use crate::bot_data::BotData;

use tokio::fs;

use once_cell::sync::Lazy;

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

macro_rules! reply {
    ($ctx: ident, $msg: ident, $say: expr) => {
        reply!($ctx, $msg, $say, false)
    };

    ($ctx: ident, $msg: ident, $say: expr, $ping: expr) => {
        if let Err(why) = $crate::util::send_reply(&$ctx, &$msg, $say, $ping).await {
            println!("Could not send reply: {:?}", why);
        }
    };
    
}

macro_rules! long_say {
    ($ctx: ident, $msg: ident, $say: expr, $sep: expr) => {
        if let Err(why) = $crate::util::send_long_message(&$ctx, &$msg, $say, $sep).await {
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

pub(crate) async fn send_reply<T>(
    ctx: &Context,
    msg_to_reply_to: &Message,
    reply_msg: T,
    mention_author: bool
) -> Result<Message, serenity::Error>
where
    T: std::fmt::Display,
{

    if mention_author {
        msg_to_reply_to.reply_ping(&ctx, reply_msg).await
    } else {
        msg_to_reply_to.reply(&ctx, reply_msg).await
    }
}

pub(crate) fn _build_time_rem(now: i64, end: i64) -> String {
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

pub(crate) async fn edit_message_by_id<T: ToString, S: Into<ChannelId>, V: Into<MessageId>>(
    cache_http: impl CacheHttp,
    channel_id: S,
    message_id: V,
    new_msg: T,
) -> Result<Message, serenity::Error> {
    let channel: ChannelId = channel_id.into();

    channel
        .edit_message(cache_http.http(), message_id, |e| {
            e.content(new_msg.to_string())
        })
        .await

    // let mut edited_text = EditMessage::default();
    // edited_text.content(new_msg.to_string());
    // let map = serenity::utils::hashmap_to_json_map(edited_text.0);
    // let stringified_map = serde_json::Value::Object(map);
    // cache_http
    // .http()
    // .edit_message(channel_id, message_id, &stringified_map)
    // .await
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

    let current_user_id = ctx.cache.current_user_id().await;

    let permissions = guild_channel
        .permissions_for_user(ctx, current_user_id)
        .await
        .unwrap();

    permissions.contains(Permissions::ADD_REACTIONS | Permissions::MANAGE_MESSAGES)
}

// pub fn to_boxed_fut<F>(fut: impl Fn(Arc<Context>, Arc<Message>, Args) -> F) -> Pin<Box<dyn std::future::Future<Output = CommandResult>>>
// where F: std::future::Future<Output=CommandResult> {
// Box::pin(fut)
// }


const NUMBERS : Lazy<Vec<ReactionType>> = Lazy::new(|| {
    vec![
        ReactionType::Unicode("\u{31}\u{fe0f}\u{20e3}".into()), // 1
        ReactionType::Unicode("\u{32}\u{fe0f}\u{20e3}".into()),
        ReactionType::Unicode("\u{33}\u{fe0f}\u{20e3}".into()),
        ReactionType::Unicode("\u{34}\u{fe0f}\u{20e3}".into()),
        ReactionType::Unicode("\u{35}\u{fe0f}\u{20e3}".into()),
        ReactionType::Unicode("\u{36}\u{fe0f}\u{20e3}".into()),
        ReactionType::Unicode("\u{37}\u{fe0f}\u{20e3}".into()),
        ReactionType::Unicode("\u{38}\u{fe0f}\u{20e3}".into()),
        ReactionType::Unicode("\u{39}\u{fe0f}\u{20e3}".into()),
    ]
});

/*
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
*/

const REACTION_TIMEOUT: Duration = Duration::from_secs(30);

/// Panics if len is 0 or greater than 9
pub(crate) async fn reaction_did_you_mean(
    ctx: &Context,
    msg: &Message,
    author_id: UserId,
    len: usize,
) -> Option<usize> {
    
    if len == 0 || len > 9 {
        panic!("Recieved invalid number for did you mean: {}", len);
    }

    if !has_reaction_perm(&ctx, msg.channel_id).await {
        return None;
    }

    let http_clone = Arc::clone(&ctx.http);
    let msg_id = msg.id.0;
    let channel_id = msg.channel_id.0;
    let all_reactions_added = tokio::spawn(add_reactions(http_clone, len, channel_id, msg_id));

    let mut reacted_number: Option<usize> = None;

    // using a closure here just to make code cleaner looking
    // optimizer probably inlines anyway
    let reaction_collector = || { 
        msg.await_reaction(
            &ctx
        ).timeout(
            REACTION_TIMEOUT
        ).author_id(
            author_id
        )
    };
    
    while let Some(reaction) = reaction_collector().await {
        let emoji = &reaction.as_inner_ref().emoji;
        let res = get_number_pos(emoji, len);
        if let Some(num) = res {
            reacted_number = Some(num);
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

fn get_number_pos(reaction: &ReactionType, len: usize) -> Option<usize> {
    for (emoji, num) in NUMBERS.iter().zip(0..=len) {
        if emoji == reaction {
            return Some(num)
        }
    }
    None
}

async fn add_reactions(http: Arc<Http>, len: usize, channel_id: u64, msg_id: u64) -> bool {
    for number in NUMBERS.iter().take(len) {
        if let Err(why) = http
            .create_reaction(
                channel_id,
                msg_id,
                number,
            )
            .await
        {
            println!("Could not react to message: {:?}", why);
            return false;
        }
    }
    true
}

#[command]
/// Get the last few lines of the server log file
pub(crate) async fn audit(ctx: &Context, msg: &Message, _: Args) -> CommandResult {

    let res = fs::read_to_string("./nohup.out").await;
    match res {
        Ok(val) => {
            let lines: Vec<&str> = val.split('\n').filter(|&i| !i.trim().is_empty()).collect();
            
            if lines.len() == 0 {
                say!(ctx, msg, "Log is empty");
            } else if lines.len() < 11 {
                long_say!(ctx, msg, &lines, "\n");
            } else {
                let len = lines.len() - 11;
                long_say!(ctx, msg, &lines[len..], "\n");
            }
        }
        Err(err) => {
            say!(ctx, msg, format!("unable to get log, {}", err.to_string()));
        }
    }
    Ok(())
}

#[command]
/// Get a link to the BnB Battlechip manager website
async fn manager(ctx: &Context, msg: &Message, _: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let config = data.get::<BotData>().expect("could not get config");
    reply!(ctx, msg, &config.manager);
    Ok(())
}

#[command]
/// Get a link to the BnB Players Handbook
async fn phb(ctx: &Context, msg: &Message, _: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let config = data.get::<BotData>().expect("could not get config");
    reply!(ctx, msg, &config.phb);
    Ok(())
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
    Ok(())
}

#[command]
/// Tests the length of time it takes the bot to send a message
async fn ping(ctx: &Context, msg: &Message, _: Args) -> CommandResult {
    let instant = std::time::Instant::now();

    let res = send_reply(ctx, msg, "Loading response times, please wait...", false).await;

    let duration = instant.elapsed();
    
    let mut to_edit = match res {
      Ok(to_reply) => to_reply,
      Err(why) => {
        println!("Could not send message: {:?}", why);
        return Ok(());
      }
    };

    let ms = (duration.as_micros() as f64) / 1000f64;

    let new_text = format!("\u{1F3D3} Pong!, that took {:.2} ms", ms);

    let res = to_edit.edit(&ctx, |m| {
        m.content(&new_text)
    }).await;

    if let Err(why) = res {
        println!("Could not edit message: {:?}", why);
    }

    Ok(())
}

#[command]
async fn groups(ctx: &Context, msg: &Message, _: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let bot_data = data.get::<BotData>().expect("No bot data");
    let res = reqwest::get(&bot_data.groups_url).await;
    let error_msg = "Error occurred checking open folder groups, please try again later. If the problem persists inform Major";
    let resp = match res {
        Ok(resp) => resp,
        Err(why) => {
            eprintln!("Failed to make reqwest to get groups\n{:?}", why);
            reply!(ctx, msg, error_msg);
            return Ok(());
        }
    };

    let resp_code = resp.status().as_u16();

    // 204 is the empty response code
    if resp_code == 204 {
        reply!(ctx, msg, "There are currently no open folder groups.");
    } else if resp_code == 200 {
        let data_res = resp.json::<Vec<String>>().await;
        match data_res {
            Ok(val) => {
                let text = val.join(", ");
                reply!(ctx, msg, format!("The groups currently open are:\n```{}```", text));
            }
            Err(why) => {
                eprintln!("Failed to deserialize response\n{:?}", why);
                reply!(ctx, msg, error_msg);
            }
        }
    } else {
        eprintln!("Unknown response code {} recieved", resp_code);
        reply!(ctx, msg, error_msg);
    }

    Ok(())
}

static SHOULD_DM_OWNER: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(true));

pub(crate) async fn dm_owner<T>(
    ctx: &Context,
    to_send: T,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    T: std::fmt::Display,
{
    let data = ctx.data.read().await;

    if !SHOULD_DM_OWNER.load(Ordering::Relaxed) {
        return Ok(());
    }

    let config = data.get::<BotData>().expect("no bot data, panicking");

    let owner_id = UserId::from(config.owner);

    if let Some(owner) = ctx.cache.user(&owner_id).await {
        let _ = owner.dm(ctx, |m| m.content(format!("{}", to_send))).await?;
    } else {
        let owner = ctx.http.get_user(config.owner).await?;
        let _ = owner.dm(ctx, |m| m.content(format!("{}", to_send))).await?;
    }
    Ok(())
}

#[command("shut_up")]
/// Makes the bot stop DMing the owner on certain events
async fn shut_up(ctx: &Context, msg: &Message, _: Args) -> CommandResult {
    
    SHOULD_DM_OWNER.fetch_xor(true, Ordering::Relaxed);
    
    msg.react(ctx, '\u{1f44d}').await?;
    
    Ok(())
}