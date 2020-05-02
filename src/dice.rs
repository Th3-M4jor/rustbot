use rand::{
    distributions::{Distribution, Uniform},
    rngs::ThreadRng,
};
use serenity::{
    framework::standard::{macros::*, Args, CommandResult},
    model::channel::Message,
    prelude::*,
};

pub struct DieRoll;

impl DieRoll {
    pub fn roll_dice(to_roll: &str, reroll: bool) -> (i64, Vec<i64>) {
        let mut to_ret: i64 = 0;
        let mut rolls: Vec<i64> = vec![];
        let a: Vec<&str> = to_roll.split('+').collect();
        if a.len() > 1 {
            for b in a {
                let (res, mut vec_to_push) = DieRoll::roll_dice(b, reroll);
                to_ret += res;
                rolls.append(&mut vec_to_push);
            }
        } else {
            let d: Vec<&str> = a[0].split('d').collect();
            if d.len() == 1 {
                let res = d[0].trim().parse::<i64>();

                if let Ok(val) = res {
                    rolls.push(val);
                    return (val, rolls);
                } else {
                    return (-1, vec![]);
                }
            }
            let amt_to_roll: i64;
            let res = d[0].trim().parse::<i64>();
            match res {
                Ok(val) => amt_to_roll = val,
                Err(_) => amt_to_roll = 1,
            }
            let mut rng = ThreadRng::default();
            for i in d.iter().skip(1) {
                let f: i64;
                let res = i.trim().parse::<i64>();
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
                    let mut to_add = die.sample(&mut rng);
                    if reroll && to_add < 2 {
                        let new_to_add = die.sample(&mut rng);
                        if new_to_add > to_add {
                            to_add = new_to_add;
                        }
                    }
                    rolls.push(to_add);
                    u += to_add;
                }
                to_ret += u;
            }
        }

        (to_ret, rolls)
    }
}

#[group]
#[commands(roll, reroll, roll_stats)]
/// A group of commands related to rolling dice
struct Dice;

async fn perform_roll(ctx: &Context, msg: &Message, to_roll: &str, reroll: bool) {
    // let mut results: Vec<i64> = vec![];
    let (amt, results) = DieRoll::roll_dice(&to_roll, reroll);
    let repl_str = format!("{:?}", results);
    let reply = if repl_str.len() > 1850 {
        format!(
            "{}, you rolled: {}\n[There were too many die rolls to show the result of each one]",
            msg.author.mention(),
            amt
        )
    } else {
        format!(
            "{}, you rolled: {}\n{}",
            msg.author.mention(),
            amt,
            repl_str
        )
    };
    say!(ctx, msg, reply);
}

#[command]
/// Same as the roll command, except 1's and 2's will be re-rolled once, keeping the higher result
async fn reroll(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
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
    perform_roll(ctx, msg, args.rest(), true).await;
    return Ok(());
}

#[command]
/// Roll a number of dice, using the format XdY where X is the number of dice, and Y is the number of sides on the die to roll
#[example = "1d20"]
#[example = "4d27"]
pub(crate) async fn roll(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
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
    perform_roll(ctx, msg, args.rest(), false).await;
    return Ok(());
}

#[command("rollstats")]
/// Roll character stats for D&D 5e by rolling 4d6 and dropping the lowest 6 times
pub(crate) async fn roll_stats(ctx: &Context, msg: &Message, _: Args) -> CommandResult {
    let mut stats: [i64; 6] = [0; 6];
    // let mut rolls: Vec<i64> = vec![];
    for i in &mut stats {
        // rolls.clear();
        let (_, mut rolls) = DieRoll::roll_dice("4d6", false);

        // sort reverse to put lowest at the end
        rolls.sort_unstable_by(|a, b| b.cmp(a));
        rolls.pop();

        *i = rolls.iter().sum();
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
