use serde::{Deserialize, Serialize};
use simple_error::SimpleError;
use std::hash::Hash;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy, Hash, PartialOrd, Ord)]
pub enum Skills {
    Perception,
    Info,
    Tech,
    Strength,
    Agility,
    Endurance,
    Charm,
    Valor,
    Affinity,
    None,
    Varies,
}

impl std::str::FromStr for Skills {
    type Err = SimpleError;

    fn from_str(to_parse: &str) -> Result<Skills, SimpleError> {
        match to_parse.to_ascii_lowercase().as_str() {
            "perception" => Ok(Skills::Perception),
            "per" => Ok(Skills::Perception),
            "info" => Ok(Skills::Info),
            "inf" => Ok(Skills::Info),
            "tech" => Ok(Skills::Tech),
            "tch" => Ok(Skills::Tech),
            "strength" => Ok(Skills::Strength),
            "str" => Ok(Skills::Strength),
            "agility" => Ok(Skills::Agility),
            "agi" => Ok(Skills::Agility),
            "endurance" => Ok(Skills::Endurance),
            "end" => Ok(Skills::Endurance),
            "charm" => Ok(Skills::Charm),
            "chm" => Ok(Skills::Charm),
            "valor" => Ok(Skills::Valor),
            "vlr" => Ok(Skills::Valor),
            "affinity" => Ok(Skills::Affinity),
            "aff" => Ok(Skills::Affinity),
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
            Skills::Perception => write!(f, "Perception"),
            Skills::Info => write!(f, "Info"),
            Skills::Tech => write!(f, "Tech"),
            Skills::Strength => write!(f, "Strength"),
            Skills::Agility => write!(f, "Agility"),
            Skills::Endurance => write!(f, "Endurance"),
            Skills::Charm => write!(f, "Charm"),
            Skills::Valor => write!(f, "Valor"),
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

impl Skills {
    pub(crate) fn abbreviation(&self) -> &str {
        match self {
            Skills::Perception => "PER",
            Skills::Info => "INF",
            Skills::Tech => "TCH",
            Skills::Strength => "STR",
            Skills::Agility => "AGI",
            Skills::Endurance => "END",
            Skills::Charm => "CHM",
            Skills::Valor => "VLR",
            Skills::Affinity => "AFF",
            Skills::None => "--",
            Skills::Varies => "VAR",
        }
    }
}