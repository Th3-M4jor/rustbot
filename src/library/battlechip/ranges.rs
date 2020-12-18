use serde::{Deserialize, Serialize};
use simple_error::SimpleError;

#[derive(Deserialize, Serialize)]
pub enum Ranges {
    #[serde(rename(serialize = "Self"))]
    Itself,
    Close,
    Near,
    Far,
    Varies,
}

impl std::str::FromStr for Ranges {
    type Err = SimpleError;

    fn from_str(to_parse: &str) -> Result<Ranges, SimpleError> {
        match to_parse.to_ascii_lowercase().as_str() {
            "self" => Ok(Ranges::Itself),
            "close" => Ok(Ranges::Close),
            "near" => Ok(Ranges::Near),
            "far" => Ok(Ranges::Far),
            "varies" => Ok(Ranges::Varies),
            _ => Err(SimpleError::new("Failed to parse range")),
        }
    }
}

impl std::fmt::Display for Ranges {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ranges::Itself => write!(f, "Self"),
            Ranges::Close => write!(f, "Close"),
            Ranges::Near => write!(f, "Near"),
            Ranges::Far => write!(f, "Far"),
            Ranges::Varies => write!(f, "Varies"),
        }
    }
}
