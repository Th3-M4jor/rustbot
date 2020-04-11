use crate::library::battlechip::chip_type::ChipType;
use crate::library::battlechip::ranges::Ranges;
use crate::library::battlechip::skills::Skills;
use crate::library::elements::Elements;
use regex::{Captures, Regex};
use serde::Serialize;
use std::cmp::{Ord, Ordering};
use std::str::FromStr;
use unicode_normalization::UnicodeNormalization;

use crate::library::LibraryObject;
use serde::export::Formatter;
use simple_error::SimpleError;

mod chip_type;
mod ranges;
pub(crate) mod skills;

#[derive(Serialize)]
#[serde(rename_all(serialize = "PascalCase"))]
pub struct BattleChip {
    pub name: String,
    pub element: Vec<Elements>,
    pub skills: Vec<Skills>,
    pub range: Ranges,
    pub damage: String,
    #[serde(rename(serialize = "Type"))]
    pub class: ChipType,
    pub hits: String,
    pub description: String,
    pub all: String,
    pub skill_target: Skills,
    pub skill_user: Skills,
}

impl Ord for BattleChip {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.to_lowercase().cmp(&other.name.to_lowercase())
    }
}

impl PartialOrd for BattleChip {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl PartialEq for BattleChip {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for BattleChip {}

impl std::fmt::Display for BattleChip {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        return write!(f, "```{}```", self.all);
    }
}

impl LibraryObject for BattleChip {
    #[inline]
    fn get_name(&self) -> &str {
        &self.name
    }
}

impl BattleChip {
    fn parse_elements(elem_str: &str) -> Result<Vec<Elements>, SimpleError> {
        let mut to_ret = vec![];
        for elem in elem_str.split(", ") {
            to_ret.push(Elements::from_str(elem)?);
        }
        to_ret.shrink_to_fit();
        Ok(to_ret)
    }

    fn parse_skills(skills_str: &str) -> Result<Vec<Skills>, SimpleError> {
        let mut to_ret = vec![];
        for skill in skills_str.split(", ") {
            to_ret.push(Skills::from_str(skill)?);
        }
        to_ret.shrink_to_fit();
        Ok(to_ret)
    }

    pub fn from_chip_string(
        first_line: &str,
        second_line: &str,
    ) -> Result<BattleChip, SimpleError> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(.+?)\s-\s(.+?)\s\|\s(.+?)\s\|\s(.+?)\s\|\s(\d+d\d+|--)\s?(?:damage)?\s?\|?\s?(Mega|Giga)?\s\|\s(\d+|\d+-\d+|--)\s?(?:hits?)\.?").expect("could not compile chip regex");
            static ref R_SAVE : Regex = Regex::new(r"an?\s(\w+)\scheck\sof\s\[DC\s\d+\s\+\s(\w+)]").expect("could not compile save regex");
        }

        //let RE : Regex = Regex::new(r"(.+?)\s-\s(.+?)\s\|\s(.+?)\s\|\s(.+?)\s\|\s(\d+d\d+|--)\s?(?:damage)?\s?\|?\s?(Mega|Giga)?\s\|\s(\d+|\d+-\d+|--)\s?(?:hits?)\.?").unwrap();
        //let R_SAVE : Regex = Regex::new(r"an?\s(\w+)\scheck\sof\s\[DC\s\d+\s\+\s(\w+)]").unwrap();
        let chip_val: Captures = RE
            .captures(first_line)
            .ok_or_else(|| SimpleError::new("Failed at capture stage"))?;

        let chip_name = chip_val
            .get(1)
            .ok_or_else(|| SimpleError::new("Could not get name"))?
            .as_str()
            .trim();
        let chip_range = Ranges::from_str(
            chip_val
                .get(4)
                .ok_or_else(|| SimpleError::new("Failed to convert range"))?
                .as_str(),
        )?;
        let chip_damage = chip_val
            .get(5)
            .ok_or_else(|| SimpleError::new("failed to get damage"))?
            .as_str();
        let chip_hits = chip_val
            .get(7)
            .ok_or_else(|| SimpleError::new("failed to get hits"))?
            .as_str();
        let chip_type: ChipType;

        if let Some(chip_type_str) = chip_val.get(6) {
            chip_type = ChipType::from_str(chip_type_str.as_str())?;
        } else {
            chip_type = ChipType::Standard;
        }

        let parsed_elements = BattleChip::parse_elements(
            chip_val
                .get(2)
                .ok_or_else(|| SimpleError::new("failed to parse element"))?
                .as_str(),
        )?;
        //let skills : Vec<&str> = chip_val.get(3).unwrap().as_str().split(", ").collect();
        let parsed_skills = BattleChip::parse_skills(
            chip_val
                .get(3)
                .ok_or_else(|| SimpleError::new("failed to parse skills"))?
                .as_str(),
        )?;

        let skill_user: Skills;
        let skill_target: Skills;
        //let skill_res = R_SAVE.captures(second_line);
        if let Some(skill_res) = R_SAVE.captures(second_line) {
            let skill_user_res = skill_res
                .get(2)
                .ok_or_else(|| SimpleError::new("failed to get skill user"))?
                .as_str();
            let skill_target_res = skill_res
                .get(1)
                .ok_or_else(|| SimpleError::new("failed to get skill target"))?
                .as_str();
            skill_user = Skills::from_str(skill_user_res).unwrap_or_default();
            skill_target = Skills::from_str(skill_target_res).unwrap_or_default();
        } else {
            skill_user = Skills::None;
            skill_target = Skills::None;
        }

        let chip_all = format!("{}\n{}", first_line, second_line);

        let to_ret = BattleChip {
            name: chip_name.nfc().collect::<String>(),
            element: parsed_elements,
            skills: parsed_skills,
            range: chip_range,
            damage: chip_damage.nfc().collect::<String>(),
            class: chip_type,
            hits: chip_hits.nfc().collect::<String>(),
            description: second_line.nfc().collect::<String>(),
            all: chip_all.nfc().collect::<String>(),
            skill_target,
            skill_user,
        };

        Ok(to_ret)
    }
}
