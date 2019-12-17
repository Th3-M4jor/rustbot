use simple_error::SimpleError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Elements {
    Fire,
    Aqua,
    Elec,
    Wood,
    Wind,
    Sword,
    Break,
    Cursor,
    Recovery,
    Invis,
    Object,
    Null,
}

impl std::str::FromStr for Elements {
    type Err = SimpleError;
    fn from_str(to_parse: &str) -> Result<Elements, SimpleError> {

        match to_parse.to_ascii_lowercase().as_str() {
            "fire" => Ok(Elements::Fire),
            "aqua" => Ok(Elements::Aqua),
            "elec" => Ok(Elements::Elec),
            "wood" => Ok(Elements::Wood),
            "wind" => Ok(Elements::Wind),
            "sword" => Ok(Elements::Sword),
            "break" => Ok(Elements::Break),
            "cursor" => Ok(Elements::Cursor),
            "recovery" => Ok(Elements::Recovery),
            "invis" => Ok(Elements::Invis),
            "object" => Ok(Elements::Object),
            "null" => Ok(Elements::Null),
            _ => Err(SimpleError::new("could not parse element")),
        }
    }
}

impl std::fmt::Display for Elements {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Elements::Fire => write!(f,"{}", "Fire"),
            Elements::Aqua => write!(f,"{}", "Aqua"),
            Elements::Elec => write!(f,"{}", "Elec"),
            Elements::Wood => write!(f,"{}", "Wood"),
            Elements::Wind => write!(f,"{}", "Wind"),
            Elements::Sword => write!(f,"{}", "Sword"),
            Elements::Break => write!(f,"{}", "Break"),
            Elements::Cursor => write!(f,"{}", "Cursor"),
            Elements::Recovery => write!(f,"{}", "Recovery"),
            Elements::Invis => write!(f,"{}", "Invis"),
            Elements::Object => write!(f,"{}", "Object"),
            Elements::Null => write!(f,"{}", "Null"),
        }
    }
}