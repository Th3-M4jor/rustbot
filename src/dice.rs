use rand::distributions::{Distribution, Uniform};
use rand::rngs::ThreadRng;
use serenity::{model::channel::Message, prelude::*};
use serenity::framework::standard::{macros::command, Args, CommandResult};
use std::borrow::BorrowMut;
use crate::bot_data::BotData;

pub struct DieRoll;

impl DieRoll {
    pub fn roll_dice(to_roll: &str, rolls: &mut Vec<i64>, reroll: bool) -> i64 {
        let mut to_ret: i64 = 0;
        let a: Vec<&str> = to_roll.split('+').collect();
        if a.len() > 1 {
            for b in a {
                to_ret += DieRoll::roll_dice(b, rolls, reroll);
            }
        } else {
            let d: Vec<&str> = a[0].split('d').collect();
            if d.len() == 1 {
                let res = d[0].trim().parse::<i64>();

                match res {
                    Ok(val) => {
                        rolls.push(val);
                        return val;
                    }
                    Err(_) => {
                        return 0;
                    }
                }
            }
            let amt_to_roll: i64;
            let res = d[0].trim().parse::<i64>();
            match res {
                Ok(val) => amt_to_roll = val,
                Err(_) => amt_to_roll = 1,
            }
            let mut rng = ThreadRng::default();
            for i in 1..d.len() {
                let f: i64;
                let res = d[i].trim().parse::<i64>();
                match res {
                    Ok(val) => {
                        if val <= 1 {
                            f = 6;
                        } else {
                            f = val;
                        }
                    }
                    Err(_) => f = 6,
                }
                let die = Uniform::from(1..=f);
                let mut u: i64 = 0;
                for _ in 0..amt_to_roll {
                    //let to_add = rng.gen::<i64>().abs() % f + 1;
                    let mut to_add = die.sample(&mut rng);
                    if reroll && to_add < 2 {
                        let new_to_add = die.sample(&mut rng);
                        if new_to_add > to_add {to_add = new_to_add;}
                    }
                    rolls.push(to_add);
                    u += to_add;
                }
                to_ret += u;
            }
        }

        return to_ret;
    }
}

#[command]
#[aliases("reroll")]
pub(crate) fn roll(ctx: &mut Context, msg: &Message, _: Args) -> CommandResult {
    let mut args: Vec<&str>;
    let new_first;

    {
        let data = ctx.data.read();
        let config = data.get::<BotData>().expect("no config found");
        //msg_content_clone = msg.content.clone();
        args = msg.content.split(" ").collect();
        new_first = args[0].replacen(&config.cmd_prefix, "", 1);
        args[0] = new_first.as_str();
    }
    
    
    if args.len() < 2 {
        say!(
            ctx,
            msg,
            format!(
                "{}, you must supply a number of dice to roll",
                msg.author.mention()
            )
        );
        return Ok(());
    }

    

    //grab all but the first argument which is the command name
    let to_join = &args[1..];

    let to_roll = to_join.join(" ");
    let mut results: Vec<i64> = vec![];
    let amt;
    if args[0] == "reroll" {
        amt = DieRoll::roll_dice(&to_roll, results.borrow_mut(), true);
    } else {
        amt = DieRoll::roll_dice(&to_roll, results.borrow_mut(), false);
    }
    let repl_str = format!("{:?}", results);
    let reply;
    if repl_str.len() > 1850 {
        reply = format!(
            "{}, you rolled: {}\n[There were too many die rolls to show the result of each one]",
            msg.author.mention(),
            amt
        );
    } else {
        reply = format!(
            "{}, you rolled: {}\n{}",
            msg.author.mention(),
            amt,
            repl_str
        );
    }
    say!(ctx, msg, reply);
    return Ok(());
}

#[command]
#[aliases("rollstats")]
pub(crate) fn roll_stats(ctx: &mut Context, msg: &Message, _: Args) -> CommandResult {
    let mut stats: [i64; 6] = [0; 6];
    let mut rolls: Vec<i64> = vec![];
    for i in &mut stats {
        rolls.clear();
        DieRoll::roll_dice("4d6", &mut rolls, false);

        //sort reverse to put lowest at the end
        rolls.sort_unstable_by(|a, b| b.cmp(a));
        rolls.pop();

        *i = rolls.iter().fold(0, |acc, val| acc + val);
    }

    say!(
        ctx,
        msg,
        format!(
            "{}, 4d6 drop the lowest:\n{:?}",
            msg.author.mention(),
            stats
        )
    );
    return Ok(());
}
