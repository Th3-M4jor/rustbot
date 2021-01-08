use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    prelude::*,
};

use serde_json::json;

use crate::library::blights::Panels;

#[cfg(debug_assertions)]
use crate::bot_data::BotData;


#[command]
async fn create_commands(ctx: &Context, _: &Message, _: Args) -> CommandResult {

    if let Err(why) = create_panel_cmd(ctx).await {
        eprintln!("Error creating panel cmd, {:?}", why);
        return Ok(());
    }

    if let Err(why) = create_roll_cmd(ctx).await {
        eprintln!("Error creating roll cmd, {:?}", why);
        return Ok(());
    }

    Ok(())
}

async fn create_panel_cmd(ctx: &Context) -> Result<(), serenity::Error> {

    let data = ctx.data.read().await;
    let panel_lock = data.get::<Panels>().expect("panels not found");
    let panel_list = panel_lock.read().await;

    let panels = panel_list.to_slash_opts().expect("Panels don't exist");
    

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