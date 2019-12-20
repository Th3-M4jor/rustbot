
use simple_error::SimpleError;

pub fn get_levenshtein(first: &str, second: &str) -> Result<usize, SimpleError> {
    if first.is_empty() || second.is_empty() {
        return Err(SimpleError::new("empty string was in test"));
    }
    /*
    int n = s.Length; // length of s
            int m = t.Length; // length of t

            if (n == 0)
            {
                return m;
            }

            if (m == 0)
            {
                return n;
            }
    */
    let n = first.len();
    let m = second.len();
    if n == 0 {
        return Ok(m);
    }
    if m == 0 {
        return Ok(n);
    }
    /*
     int[] p = new int[n + 1]; //'previous' cost array, horizontally
     int[] d = new int[n + 1]; // cost array, horizontally
    */
    let mut p : Vec<usize> = vec![];
    let mut d : Vec<usize> = vec![];
    p.resize(n + 1, 0);
    d.resize(n + 1, 0);



    return Ok(0);
}