use rand::rngs::ThreadRng;
use rand::Rng;

pub struct DieRoll;

impl DieRoll {
    pub fn roll_dice(to_roll: &str, rolls: &mut Vec<i64>) -> i64 {
        let mut to_ret : i64 = 0;
        let a : Vec<&str> = to_roll.split('+').collect();
        if a.len() > 1 {
            for b in a {
                to_ret += DieRoll::roll_dice(b, rolls);
            }
        } else {
            let d : Vec<&str> = a[0].split('d').collect();
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
            let amt_to_roll : i64;
            let res = d[0].trim().parse::<i64>();
            match res {
                Ok(val) => amt_to_roll = val,
                Err(_) => amt_to_roll = 1,
            }
            let mut rng = ThreadRng::default();
            for i in 1..d.len() {
                let f : i64;
                let res = d[i].trim().parse::<i64>();
                match res {
                    Ok(val) => f = val,
                    Err(_) => f = 6,
                }
                let mut u :i64 = 0;
                for _ in 0..amt_to_roll {
                    let to_add = rng.gen::<i64>().abs() % f + 1;
                    rolls.push(to_add);
                    u += to_add;
                }
                to_ret += u;
            }


        }

        return to_ret;
    }

}