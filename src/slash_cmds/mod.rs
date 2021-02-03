use serde_json::json;
use serenity::{
    prelude::*,
    model::interactions::*
};

use crate::{
    library::blights::{
        Panels,
        Blights,
        Statuses,
    },
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

        "blight" => blight_command(ctx, data).await,
        "panels" => panel_command(ctx, data).await,
        "roll" => roll_command(data).await,
        "shuffle" => shuffle_command(data).await,
        "status" => status_command(ctx, data).await,
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
    let panel_opt = data.options.get(0).and_then(
        |d| d.value.as_ref()
    ).and_then(|p| p.as_str());

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

async fn roll_command(data: &ApplicationCommandInteractionData) -> serde_json::Value {
    let to_roll = data.options.get(0).and_then(|d| d.value.as_ref()).and_then(|o| o.as_str()).unwrap_or("1d20");
    let owned_to_roll = to_roll.to_owned();
    let result = tokio::task::spawn_blocking(move || {
        let (amt, results) = DieRoll::roll_dice(&owned_to_roll, false)?;

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

        Some(json!({
            "type": 4,
            "data": {
                "content": reply
            }
        }))

    }).await;

    match result {
        Ok(Some(val)) => val,
        Ok(None) => {
            eprintln!("Too man die rolls: {}", to_roll);
            json!({
                "type": 4,
                "data": {
                    "content": "An error occurred while rolling, too many dice maybe?"
                }
            })
        }
        Err(why) => {
            eprintln!("Spawn Blocking panicked\n{:?}", why);
            json!({
                "type": 4,
                "data": {
                    "content": "An error occurred while rolling, too many dice maybe?"
                }
            })
        }
    }

}

async fn shuffle_command(data: &ApplicationCommandInteractionData) -> serde_json::Value {
    let to_shuffle_opt = data.options.get(0).and_then(|d| d.value.as_ref()).and_then(|o| o.as_u64());
    let val = match to_shuffle_opt {
        Some(val) => val,
        None => {
            return json!({
                "type": 4,
                "data": {
                    "content": "Unable to parse number to shuffle, inform Major"
                }
            });
        }
    };

    let res = tokio::task::spawn_blocking(move || crate::dice::perform_shuffle(val as usize)).await;

    match res {
        Ok(reply) => {
            json!({
                "type": 4,
                "data": {
                    "content": reply
                }
            })
        }
        Err(why) => {
            eprintln!("Thread panicked, {:?}", why);
            json!({
                "type": 4,
                "data": {
                    "content": "Error occurred while shuffling, too many numbers maybe?"
                },
            })
        }
    }


}

async fn blight_command(ctx: &Context, data: &ApplicationCommandInteractionData) -> serde_json::Value {

    let blight_opt = data.options.get(0).and_then(
        |d| d.value.as_ref()
    ).and_then(|b| b.as_str());

    let blight = match blight_opt {
        Some(blight) => blight,
        None => {
            return json!({
                "type": 4,
                "data": {
                    "content": "No blight element was provided"
                }
            });
        }
    };
    
    let data = ctx.data.read().await;
    let blight_lock = data.get::<Blights>().expect("panels not found");
    let blight_list = blight_lock.read().await;

    let to_send = match blight_list.get(blight) {
        Some(val) => {
            format!("```{}```", val)
        }
        None => format!("There is no blight of the element {}, perhaps you spelled it wrong?", blight),
    };

    json!({
        "type": 4,
        "data": {
            "content": to_send
        }
    })
}

async fn status_command(ctx: &Context, data: &ApplicationCommandInteractionData) -> serde_json::Value {
    let status_opt = data.options.get(0).and_then(
        |d| d.value.as_ref()
    ).and_then(|s| s.as_str());

    let status = match status_opt {
        Some(status) => status.to_ascii_lowercase(),
        None => {
            return json!({
                "type": 4,
                "data": {
                    "content": "No status was provided"
                }
            });
        }
    };
    
    let data = ctx.data.read().await;
    let status_lock = data.get::<Statuses>().expect("statuses not found");
    let status_list = status_lock.read().await;

    let to_send = match status_list.get(&status) {
        Some(val) => {
            format!("```{}```", val)
        }
        None => format!("There is no status of the type {}, perhaps you spelled it incorrectly?", status),
    };

    json!({
        "type": 4,
        "data": {
            "content": to_send
        }
    })
}