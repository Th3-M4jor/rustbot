use serenity::{
    async_trait,
    prelude::*,
    model::{
        event::ResumedEvent,
        gateway::{Activity, Ready},
        id::GuildId,
        interactions::Interaction,
    },
};

use std::sync::atomic::{Ordering, AtomicBool};
use once_cell::sync::Lazy;

use crate::{
    bot_data::BotData,
    util::dm_owner,
    slash_cmds::handle_interaction,
};

static FIRST_LOGIN: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(true));
static FIRST_CACHE_READY: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));

pub(crate) struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn cache_ready(&self, ctx: Context, _: Vec<GuildId>) {
        if FIRST_CACHE_READY.compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed).is_err() {
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
        if FIRST_LOGIN.compare_exchange(true, false, Ordering::AcqRel, Ordering::Relaxed).is_ok() {
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

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        handle_interaction(&ctx, &interaction).await;
    }
}