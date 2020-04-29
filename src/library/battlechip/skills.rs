use serde::{Deserialize, Serialize};
use simple_error::SimpleError;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
pub enum Skills {
    Sense,
    Info,
    Coding,
    Strength,
    Speed,
    Stamina,
    Charm,
    Bravery,
    Affinity,
    None,
    Varies,
}

impl std::str::FromStr for Skills {
    type Err = SimpleError;

    fn from_str(to_parse: &str) -> Result<Skills, SimpleError> {
        match to_parse.to_ascii_lowercase().as_str() {
            "sense" => Ok(Skills::Sense),
            "info" => Ok(Skills::Info),
            "coding" => Ok(Skills::Coding),
            "strength" => Ok(Skills::Strength),
            "speed" => Ok(Skills::Speed),
            "stamina" => Ok(Skills::Stamina),
            "charm" => Ok(Skills::Charm),
            "bravery" => Ok(Skills::Bravery),
            "affinity" => Ok(Skills::Affinity),
            "none" | "--" => Ok(Skills::None),
            //"--" => Ok(Skills::None),
            "varies" => Ok(Skills::Varies),
            _ => Err(SimpleError::new("could not parse skill")),
        }
    }
}

impl std::fmt::Display for Skills {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Skills::Sense => write!(f, "Sense"),
            Skills::Info => write!(f, "Info"),
            Skills::Coding => write!(f, "Coding"),
            Skills::Strength => write!(f, "Strength"),
            Skills::Speed => write!(f, "Speed"),
            Skills::Stamina => write!(f, "Stamina"),
            Skills::Charm => write!(f, "Charm"),
            Skills::Bravery => write!(f, "Bravery"),
            Skills::Affinity => write!(f, "Affinity"),
            Skills::None => write!(f, "--"),
            Skills::Varies => write!(f, "Varies"),
        }
    }
}

impl std::default::Default for Skills {
    fn default() -> Self {
        Skills::None
    }
}
