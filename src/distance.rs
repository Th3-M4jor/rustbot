
use simple_error::SimpleError;

pub fn get_damereau_levenshtein_distance(first: &str, second: &str) -> Result<usize, SimpleError> {
    if first.is_empty() || second.is_empty() {
        return Err(SimpleError::new("empty string was in test"));
    }
    let s : Vec<char> = first.chars().collect();
    let t : Vec<char> = second.chars().collect();

    let n = s.len();
    let m = t.len();
    if n == 0 {
        return Ok(m);
    }
    if m == 0 {
        return Ok(n);
    }

    let mut p : Vec<usize> = vec![];
    let mut d : Vec<usize> = vec![];
    p.resize(n + 1, 0);
    d.resize(n + 1, 0);
    for i in 0..p.len() {
        p[i] = i;
    }

    for j in 1..(m + 1) {
        let t_j = t[j - 1];
        d[0] = j;
        for i in 1..(n + 1) {
            let cost : usize;
            if s[i - 1] == t_j { cost = 0;} else {cost = 1;}
            let test1 = d[i - 1] + 1;
            let test2 = p[i] + 1;
            let test3 = p[i - 1] + cost;
            d[i] = std::cmp::min(std::cmp::min(test1, test2), test3);
        }
        let mut d_placeholder = p;
        p = d;
        d = d_placeholder;

    }
    return Ok(p[n]);

    //return Ok(0);
}