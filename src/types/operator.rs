use std::fs;

use super::tag::TagType;
use getset::Getters;
use serde::{self, Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum Rarity {
    #[serde(rename = "TIER_1")]
    Tier1,
    #[serde(rename = "TIER_2")]
    Tier2,
    #[serde(rename = "TIER_3")]
    Tier3,
    #[serde(rename = "TIER_4")]
    Tier4,
    #[serde(rename = "TIER_5")]
    Tier5,
    #[serde(rename = "TIER_6")]
    Tier6,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum Position {
    #[serde(rename = "MELEE")]
    Melee,
    #[serde(rename = "RANGED")]
    Ranged,
}

#[derive(Debug, Deserialize, Serialize, Getters)]
#[getset(get = "pub")]
pub struct Operator {
    id: String,
    name: String,
    rarity: Rarity,
    #[serde(default)]
    tag_list: Vec<TagType>,
    position: Position,
}

impl Operator {
    pub fn matches_tags(&self, tags: &Vec<TagType>) -> bool {
        if self.rarity == Rarity::Tier6 && !tags.contains(&TagType::TopOperator) {
            return false;
        }
        tags.iter().all(|tag| self.tag_list.contains(tag))
    }
}

pub fn load_operator_data() -> Result<Vec<Operator>, std::io::Error> {
    let file = fs::File::open("data/pool.json")?;
    let reader = std::io::BufReader::new(file);
    let var: Vec<Operator> =
        serde_json::from_reader(reader).map_err(|e: serde_json::Error| std::io::Error::from(e))?;
    // var.iter().filter(predicate)
    Ok(var)
}
