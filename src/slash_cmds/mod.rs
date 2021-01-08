use serde_json::json;
use serenity::{
    prelude::*,
    model::interactions::*
};

use crate::{
    library::blights::Panels,
    dice::DieRoll,
};

pub(crate) mod create;

pub(crate) async fn handle_interaction(ctx: &Context, interaction: &Interaction) {
    if interaction.kind == InteractionType::Ping {
        if let Err(why) = handle_interaction_ping(ctx, interaction).await {
            eprintln!("Error responding to a ping, {:?}", why);
        }
    } else if let Some(data) = &interaction.data {
        if let Err(why) = handle_command_call(ctx, interaction, data).await {
            eprintln!("Error responding to a command, {:?}", why);
        }
    }
}

async fn handle_interaction_ping(ctx: &Context, interaction: &Interaction) -> Result<(), serenity::Error> {
    //1 is the opcode for responding to a ping
    let payload = json!({
        "type": 1,
    });

    ctx.http.create_interaction_response(interaction.id.0, &interaction.token, &payload).await
}

async fn handle_command_call(ctx: &Context, interaction: &Interaction, data: &ApplicationCommandInteractionData) -> Result<(), serenity::Error> {

    let resp = match data.name.as_str() {

        "panels" => panel_command(ctx, data).await,
        "roll" => roll_command(data),
        _ => {
            eprintln!("unknown slash command used with name: {}", data.name);
            json!({
                "type": 4,
                "data": {
                    "content": "This command is no longer recognized, inform Major"
                }
            })
        }

    };

    ctx.http.create_interaction_response(interaction.id.0, &interaction.token, &resp).await?;

    Ok(())
}

async fn panel_command(ctx: &Context, data: &ApplicationCommandInteractionData) -> serde_json::Value {
    let panel_opt = match data.options.get(0).and_then(|d| d.value.as_ref()) {
        Some(panel) => panel.as_str(),
        None => {
            // 4 is the type that means show command and show response message
            return json!({
                "type": 4,
                "data": {
                    "content": "No panel type was provided"
                }
            });
        }
    };

    let panel = match panel_opt {
        Some(panel) => panel,
        None => {
            return json!({
                "type": 4,
                "data": {
                    "content": "No proper panel type was provided"
                }
            });
        }
    };

    
    let data = ctx.data.read().await;
    let panel_lock = data.get::<Panels>().expect("panels not found");
    let panel_list = panel_lock.read().await;

    let to_send = match panel_list.get(panel) {
        Some(val) => {
            format!("```{}```", val)
        }
        None => format!("There is no panel with the name {}, please inform Major of the issue", panel)
    };

    json!({
        "type": 4,
        "data": {
            "content": to_send
        }
    })
}

fn roll_command(data: &ApplicationCommandInteractionData) -> serde_json::Value {
    let to_roll = data.options.get(0).and_then(|d| d.value.as_ref()).and_then(|o| o.as_str()).unwrap_or("1d20");

    let (amt, results) = DieRoll::roll_dice(to_roll, false);

    let repl_str = format!("{:?}", results);
    let reply = if repl_str.len() > 1850 {
        format!(
            "You rolled: {}\n[There were too many die rolls to show the result of each one]",
            amt
        )
    } else {
        format!(
            "You rolled: {}\n{}",
            amt,
            repl_str
        )
    };

    json!({
        "type": 4,
        "data": {
            "content": reply
        }
    })

}