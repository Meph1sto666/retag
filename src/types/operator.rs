use std::{cmp::Ordering, fs, sync::Arc};
use super::tag::TagType;
use eframe::egui::ColorImage;
use getset::Getters;
use image::DynamicImage;
use serde::{self, Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq, PartialOrd)]
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
    #[serde(deserialize_with="deserialize_img")]
    #[serde(skip_serializing)]
    avatar: Arc<eframe::egui::ColorImage>
}

fn deserialize_img<'de, D>(deserializer: D) -> Result<Arc<eframe::egui::ColorImage>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let avatar_path: Option<String> = Option::deserialize(deserializer)?;

    let path = format!("data/assets/charavatars/{}.png", avatar_path.unwrap_or_else(|| "default_avatar".to_string()));

    let dyn_img = image::ImageReader::open(std::path::Path::new(&path))
        .map_err(serde::de::Error::custom)?
        .decode()
        .map_err(serde::de::Error::custom)?;

    let rgba_image = dyn_img.to_rgba8();
    let (width, height) = rgba_image.dimensions();
    
    let color_image = ColorImage::from_rgba_unmultiplied(
        [width as usize, height as usize],
        rgba_image.as_raw(),
    );


    Ok(Arc::new(color_image))
}

impl Operator {
    pub fn matches_tags(&self, tags: &Vec<TagType>) -> bool {
        if self.rarity == Rarity::Tier6 && !tags.contains(&TagType::TopOperator) {
            return false;
        }
        tags.iter().all(|tag| self.tag_list.contains(tag))
    }
}

pub fn load_operator_data() -> Result<Arc<Vec<Operator>>, std::io::Error> {
    let file = fs::File::open("data/pool.json")?;
    let reader = std::io::BufReader::new(file);
    let var: Vec<Operator> =
        serde_json::from_reader(reader).map_err(|e: serde_json::Error| std::io::Error::from(e)).unwrap();
    Ok(Arc::new(var))
}
