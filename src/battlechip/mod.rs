


use serde::{Deserialize, Serialize, Serializer};
use regex::{Regex, Captures};
use crate::battlechip::elements::Elements;
use crate::battlechip::skills::Skills;
use std::str::FromStr;
use std::cmp::Ordering;
use unicode_normalization::UnicodeNormalization;

use simple_error::SimpleError;
use serde::export::Formatter;
use serde::export::fmt::Error;

mod elements;
mod skills;

#[derive(Deserialize)]
pub enum Ranges {
    Itself,
    Close,
    Near,
    Far,
}

impl std::str::FromStr for Ranges {
    type Err = SimpleError;
    fn from_str(to_parse: &str) -> Result<Ranges, SimpleError> {
        match to_parse.to_ascii_lowercase().as_str() {
            "self" => Ok(Ranges::Itself),
            "close" => Ok(Ranges::Close),
            "near" => Ok(Ranges::Near),
            "far" => Ok(Ranges::Far),
            _ => Err(SimpleError::new("Failed to parse range")),
        }
    }
}

impl std::fmt::Display for Ranges {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ranges::Itself => write!(f, "{}", "Self"),
            Ranges::Close => write!(f, "{}", "Close"),
            Ranges::Near => write!(f, "{}", "Near"),
            Ranges::Far => write!(f, "{}", "Far"),
        }
    }
}

impl Serialize for Ranges {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        serializer.serialize_str(format!("{}", self).as_str())
    }
}

#[derive(Serialize, Deserialize)]
pub enum ChipType {
    Standard,
    Mega,
    Giga,
    Dark,
}

impl std::str::FromStr for ChipType {
    type Err = SimpleError;
    fn from_str(s: &str) -> Result<ChipType, SimpleError> {
        match s.to_ascii_lowercase().as_str() {
            "standard" => Ok(ChipType::Standard),
            "mega" => Ok(ChipType::Mega),
            "giga" => Ok(ChipType::Giga),
            "dark" => Ok(ChipType::Dark),
            _ => Err(SimpleError::new("Failed to parse chip type")),
        }
    }
}

impl std::fmt::Display for ChipType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChipType::Standard => write!(f,"{}", ""),
            ChipType::Mega => write!(f,"{}", "Mega"),
            ChipType::Giga => write!(f,"{}", "Giga"),
            ChipType::Dark => write!(f, "{}", "Dark"),
        }
    }
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct BattleChip {
    pub Name: String,
    pub Elements: Vec<Elements>,
    pub Skills: Vec<Skills>,
    pub Range: Ranges,
    pub Damage: String,
    pub r#Type: ChipType,
    pub Hits: String,
    pub Description: String,
    pub All: String,
    pub SkillTarget: Skills,
    pub SkillUser: Skills,
}

impl Ord for BattleChip {
    fn cmp(&self, other: &Self) -> Ordering {
        self.Name.to_lowercase().cmp(&other.Name.to_lowercase())
    }
}

impl PartialOrd for BattleChip {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl PartialEq for BattleChip {
    fn eq(&self, other: &Self) -> bool {
        self.Name == other.Name
    }
}

impl Eq for BattleChip {}

impl std::fmt::Display for BattleChip {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        return write!(f, "```{}```", self.All);
    }
}

impl BattleChip {
    pub fn new<T: Into<String>>(
        name: T,
        elements: Vec<Elements>,
        skills: Option<Vec<Skills>>,
        range: Ranges,
        damage: T,
        r#type: Option<ChipType>,
        hits: T,
        description: T,
        all: T,
        skill_target: Option<Skills>,
        skill_user: Option<Skills>,
    ) -> BattleChip {
        BattleChip {
            Name: name.into().nfc().collect::<String>(),
            Elements: elements,
            Skills: skills.unwrap_or(std::vec![Skills::None]),
            Range: range,
            Damage: damage.into().nfc().collect::<String>(),
            r#Type: r#type.unwrap_or(ChipType::Standard),
            Hits: hits.into().nfc().collect::<String>(),
            Description: description.into().nfkd().collect::<String>(),
            All: all.into().nfc().collect::<String>(),
            SkillTarget: skill_target.unwrap_or(Skills::None),
            SkillUser: skill_user.unwrap_or(Skills::None),
        }
    }

    fn parse_elements(elem_str: &str) -> Result<Vec<Elements>, SimpleError> {
        let mut to_ret = vec![];
        for elem in elem_str.split(", ") {
            to_ret.push(Elements::from_str(elem)?);
        }
        to_ret.shrink_to_fit();
        return Ok(to_ret);
    }

    fn parse_skills(skills_str: &str) -> Result<Vec<Skills>, SimpleError> {
        let mut to_ret = vec![];
        for skill in skills_str.split(", ") {
            to_ret.push(Skills::from_str(skill)?);
        }
        to_ret.shrink_to_fit();
        return Ok(to_ret);
    }

    pub fn from_chip_string(first_line: &str, second_line: &str) -> Result<Box<BattleChip>, SimpleError> {

        lazy_static! {
            static ref RE: Regex = Regex::new(r"(.+?)\s-\s(.+?)\s\|\s(.+?)\s\|\s(.+?)\s\|\s(\d+d\d+|--)\s?(?:damage)?\s?\|?\s?(Mega|Giga)?\s\|\s(\d+|\d+-\d+|--)\s?(?:hits?)\.?").expect("could not compile chip regex");
            static ref R_SAVE : Regex = Regex::new(r"an?\s(\w+)\scheck\sof\s\[DC\s\d+\s\+\s(\w+)]").expect("could not compile save regex");
        }

        //let RE : Regex = Regex::new(r"(.+?)\s-\s(.+?)\s\|\s(.+?)\s\|\s(.+?)\s\|\s(\d+d\d+|--)\s?(?:damage)?\s?\|?\s?(Mega|Giga)?\s\|\s(\d+|\d+-\d+|--)\s?(?:hits?)\.?").unwrap();
        //let R_SAVE : Regex = Regex::new(r"an?\s(\w+)\scheck\sof\s\[DC\s\d+\s\+\s(\w+)]").unwrap();
        let chip_val: Captures = RE.captures(first_line).ok_or(SimpleError::new("Failed at capture stage"))?;

        let chip_name = chip_val.get(1).ok_or(SimpleError::new("Could not get name"))?.as_str().trim();
        let chip_range = Ranges::from_str(chip_val.get(4).ok_or(SimpleError::new("Failed to convert range"))?.as_str())?;
        let chip_damage = chip_val.get(5).ok_or(SimpleError::new("failed to get damage"))?.as_str();
        let chip_hits = chip_val.get(7).ok_or(SimpleError::new("failed to get hits"))?.as_str();
        let chip_type: ChipType;
        if chip_val.get(6).is_some() {
            chip_type = ChipType::from_str(chip_val.get(6).ok_or(SimpleError::new("failed to get type"))?.as_str())?;
        } else {
            chip_type = ChipType::Standard;
        }

        let parsed_elements = BattleChip::parse_elements(chip_val.get(2)
                            .ok_or(SimpleError::new("failed to parse element"))?.as_str())?;
        //let skills : Vec<&str> = chip_val.get(3).unwrap().as_str().split(", ").collect();
        let parsed_skills = BattleChip::parse_skills(chip_val.get(3)
            .ok_or(SimpleError::new("failed to parse skills"))?.as_str())?;

        let skill_user : Skills;
        let skill_target : Skills;
        let skill_res = R_SAVE.captures(second_line);
        
        if skill_res.is_none() {
            skill_user = Skills::None;
            skill_target = Skills::None;
        } else {
            let skill_res_unwrapped = skill_res.expect("Something went wrong");
            let skill_user_res = skill_res_unwrapped.get(2).ok_or(SimpleError::new("failed to get skill user"))?.as_str();
            let skill_target_res = skill_res_unwrapped.get(1).ok_or(SimpleError::new("failed to get skill target"))?.as_str();
            skill_user = Skills::from_str(skill_user_res).unwrap_or(Skills::None);
            skill_target = Skills::from_str(skill_target_res).unwrap_or(Skills::None);
        }


        let chip_all = format!("{}\n{}", first_line, second_line);

        let to_ret = Box::new(BattleChip::new(
            chip_name,
            parsed_elements,
            Option::from(parsed_skills),
            chip_range,
            chip_damage,
            Option::from(chip_type),
            chip_hits,
            second_line,
            &chip_all,
            Option::from(skill_target),
            Option::from(skill_user),
        ));

        return Ok(to_ret);
    }

}
