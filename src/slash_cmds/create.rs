use serenity::{
    //builder::CreateInteraction,
    framework::standard::{macros::command, Args, CommandResult},
    model::{
        channel::Message,
        //interactions::ApplicationCommandOptionType,
    },
    prelude::*,
    //utils::hashmap_to_json_map,
};

use serde_json::json;

use crate::library::blights::{
    Panels,
    Blights,
    Statuses,
    StatusLike,
};

#[cfg(debug_assertions)]
use crate::bot_data::BotData;


#[command]
async fn create_commands(ctx: &Context, msg: &Message, _: Args) -> CommandResult {

    if let Err(why) = create_status_command(ctx).await {
        eprintln!("Error creating status cmd, {:?}", why);
        return Ok(());
    }

    if let Err(why) = create_panel_cmd(ctx).await {
        eprintln!("Error creating panel cmd, {:?}", why);
        return Ok(());
    }

    if let Err(why) = create_roll_cmd(ctx).await {
        eprintln!("Error creating roll cmd, {:?}", why);
        return Ok(());
    }

    if let Err(why) = create_blight_command(ctx).await {
        eprintln!("Error creating blight cmd, {:?}", why);
        return Ok(());
    }

    if let Err(why) = create_shuffle_cmd(ctx).await {
        eprintln!("Error creating shuffle cmd, {:?}", why);
        return Ok(());
    }

    msg.react(ctx, '\u{1f44d}').await?;

    Ok(())
}

async fn create_panel_cmd(ctx: &Context) -> Result<(), serenity::Error> {

    let data = ctx.data.read().await;
    let panel_lock = data.get::<Panels>().expect("panels not found");
    let panel_list = panel_lock.read().await;

    let panels = panel_list.to_slash_opts();
    
    let payload = json!({
        "name": "panels",
        "description": "Get info about a panel type",
        "options": [
            {
            "name": "panel_type",
            "description": "The kind of panel to get info on",
            "type": 3,
            "required": true,
            "choices": panels,
        },
        ],
    });

    let self_id = ctx.cache.current_user_id().await;
    
    #[cfg(debug_assertions)]
    {
        let config = data.get::<BotData>().expect("No bot data available");
        let guild_id = config.primary_guild;
        ctx.http.create_guild_application_command(self_id.0, guild_id, &payload).await?;
    }
    #[cfg(not(debug_assertions))]
    {
        ctx.http.create_global_application_command(self_id.0, &payload).await?;
    }


    Ok(())

}

async fn create_roll_cmd(ctx: &Context) -> Result<(), serenity::Error> {
    
    let payload = json!({
        "name": "roll",
        "description": "Roll XdY dice, 1d20 by default",
        "options": [
            {
            "name": "dice",
            "description": "Must be in the format XdY with optional modifiers",
            "type": 3,
            "required": false,
        },
        ],
    });
    
    let self_id = ctx.cache.current_user_id().await;
    
    #[cfg(debug_assertions)]
    {
        let data = ctx.data.read().await;
        let config = data.get::<BotData>().expect("No bot data available");
        let guild_id = config.primary_guild;
        ctx.http.create_guild_application_command(self_id.0, guild_id, &payload).await?;
    }
    #[cfg(not(debug_assertions))]
    {
        ctx.http.create_global_application_command(self_id.0, &payload).await?;
    }
    Ok(())
}

async fn create_shuffle_cmd(ctx: &Context) -> Result<(), serenity::Error> {
    let payload = json!({
        "name": "shuffle",
        "description": "shuffle a series of numbers from 1 to the given argument (inclusive)",
        "options": [
            {
                "name": "count",
                "description": "The number to shuffle",
                "type": 4,
                "required": true,
            },
        ],
    });

    let self_id = ctx.cache.current_user_id().await;

    #[cfg(debug_assertions)]
    {
        let data = ctx.data.read().await;
        let config = data.get::<BotData>().expect("No bot data available");
        let guild_id = config.primary_guild;
        ctx.http.create_guild_application_command(self_id.0, guild_id, &payload).await?;
    }
    #[cfg(not(debug_assertions))]
    {
        ctx.http.create_global_application_command(self_id.0, &payload).await?;
    }

    Ok(())
}

async fn create_status_command(ctx: &Context) -> Result<(), serenity::Error> {

    let data = ctx.data.read().await;
    let status_lock = data.get::<Statuses>().expect("Statuses unavailable");
    let statuses = status_lock.read().await;

    let status_opts = statuses.to_slash_opts();
    
    let payload = json!({
        "name": "status",
        "description": "Get info about a status type",
        "options": [
            {
            "name": "status_type",
            "description": "The kind of status to get info on",
            "type": 3,
            "required": true,
            "choices": status_opts,
        },
        ],
    });

    let self_id = ctx.cache.current_user_id().await;
    
    #[cfg(debug_assertions)]
    {
        let config = data.get::<BotData>().expect("No bot data available");
        let guild_id = config.primary_guild;
        ctx.http.create_guild_application_command(self_id.0, guild_id, &payload).await?;
    }
    #[cfg(not(debug_assertions))]
    {
        ctx.http.create_global_application_command(self_id.0, &payload).await?;
    }

    Ok(())
}

async fn create_blight_command(ctx: &Context) -> Result<(), serenity::Error> {
    
    let data = ctx.data.read().await;
    let blights_lock = data.get::<Blights>().expect("Statuses unavailable");
    let blights = blights_lock.read().await;

    let blight_opts = blights.to_slash_opts();

    let payload = json!({
        "name": "blight",
        "description": "Get info about what a blight does",
        "options": [
            {
            "name": "blight_element",
            "description": "The element to get info on the blight for",
            "type": 3,
            "required": true,
            "choices": blight_opts
        },
        ],
    });

    let self_id = ctx.cache.current_user_id().await;
    
    #[cfg(debug_assertions)]
    {
        let data = ctx.data.read().await;
        let config = data.get::<BotData>().expect("No bot data available");
        let guild_id = config.primary_guild;
        ctx.http.create_guild_application_command(self_id.0, guild_id, &payload).await?;
    }
    #[cfg(not(debug_assertions))]
    {
        ctx.http.create_global_application_command(self_id.0, &payload).await?;
    }

    Ok(())

}