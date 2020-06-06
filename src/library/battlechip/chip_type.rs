use serde::Serialize;
use simple_error::SimpleError;

#[derive(Serialize)]
pub enum ChipType {
    Standard,
    Mega,
    Giga,
    Dark,
    Support,
}

impl std::str::FromStr for ChipType {
    type Err = SimpleError;

    fn from_str(s: &str) -> Result<ChipType, SimpleError> {
        match s.to_ascii_lowercase().as_str() {
            "standard" => Ok(ChipType::Standard),
            "mega" => Ok(ChipType::Mega),
            "giga" => Ok(ChipType::Giga),
            "dark" => Ok(ChipType::Dark),
            "support" => Ok(ChipType::Support),
            _ => Err(SimpleError::new("Failed to parse chip type")),
        }
    }
}

impl std::fmt::Display for ChipType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChipType::Standard => write!(f, ""),
            ChipType::Mega => write!(f, "Mega"),
            ChipType::Giga => write!(f, "Giga"),
            ChipType::Dark => write!(f, "Dark"),
            ChipType::Support => write!(f, "Support"),
        }
    }
}

impl std::default::Default for ChipType {
    fn default() -> Self {
        ChipType::Standard
    }
}
