use serde::Serialize;
use simple_error::SimpleError;

#[derive(Serialize, PartialEq, Eq)]
pub enum ChipClass {
    Standard,
    Mega,
    Giga,
    Dark,
    Support,
}

impl std::str::FromStr for ChipClass {
    type Err = SimpleError;

    fn from_str(s: &str) -> Result<ChipClass, SimpleError> {
        match s.to_ascii_lowercase().as_str() {
            "standard" => Ok(ChipClass::Standard),
            "mega" => Ok(ChipClass::Mega),
            "giga" => Ok(ChipClass::Giga),
            "dark" => Ok(ChipClass::Dark),
            "support" => Ok(ChipClass::Support),
            _ => Err(SimpleError::new("Failed to parse chip class")),
        }
    }
}

impl std::fmt::Display for ChipClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChipClass::Standard => write!(f, ""),
            ChipClass::Mega => write!(f, "Mega"),
            ChipClass::Giga => write!(f, "Giga"),
            ChipClass::Dark => write!(f, "Dark"),
            ChipClass::Support => write!(f, "Support"),
        }
    }
}

impl std::default::Default for ChipClass {
    fn default() -> Self {
        ChipClass::Standard
    }
}

#[derive(Serialize, PartialEq, Eq)]
pub enum ChipType {
    Burst,
    Construct,
    Melee,
    Projectile,
    Wave,
    Recovery,
    Summon,
    Support,
    Trap,
}

impl ChipType {
    pub fn to_std_chip_class(&self) -> ChipClass {
        match self {
            ChipType::Trap |
            ChipType::Support |
            ChipType::Summon =>
                ChipClass::Support,
            _ => ChipClass::Standard,
        }
    }
}

impl std::str::FromStr for ChipType {
    type Err = SimpleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "burst" => Ok(ChipType::Burst),
            "construct" => Ok(ChipType::Construct),
            "melee" => Ok(ChipType::Melee),
            "projectile" => Ok(ChipType::Projectile),
            "wave" => Ok(ChipType::Wave),
            "recovery" => Ok(ChipType::Recovery),
            "summon" => Ok(ChipType::Summon),
            "support" => Ok(ChipType::Support),
            "trap" => Ok(ChipType::Trap),
            _ => Err(SimpleError::new("Failed to parse chip type")),
        }
    }
}

impl std::fmt::Display for ChipType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChipType::Burst => write!(f, "Burst"),
            ChipType::Construct => write!(f, "Construct"),
            ChipType::Melee => write!(f, "Melee"),
            ChipType::Projectile => write!(f, "Projectile"),
            ChipType::Wave => write!(f, "Wave"),
            ChipType::Recovery => write!(f, "Recovery"),
            ChipType::Summon => write!(f, "Summon"),
            ChipType::Support => write!(f, "Support"),
            ChipType::Trap => write!(f, "Trap"),
        }
    }
}
