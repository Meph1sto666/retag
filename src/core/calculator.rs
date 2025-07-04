use itertools::Itertools;
use std::sync::{Arc, Mutex};

use crate::types::{
    operator::{self, Operator, Rarity},
    tag::{TagType, UiTag},
};

#[derive(Debug)]
pub enum Order {
    Default,
}

#[derive(Debug)]
pub struct CalcResult<'a> {
    tag_variation: Vec<TagType>,
    obtainable_operators: Vec<&'a Operator>,
}

#[derive(Debug)]
pub struct Calculator {
    pool: Vec<Operator>,
    ignore_tier_1: bool,
    ignore_tier_2: bool,
    ignore_tier_3: bool,
    sort_order: Order,
}

impl Calculator {
    pub fn new() -> Calculator {
        Self {
            pool: operator::load_operator_data().unwrap_or(vec![]),
            ignore_tier_1: false,
            ignore_tier_2: false,
            ignore_tier_3: false,
            sort_order: Order::Default,
        }
    }
}

impl Calculator {
    fn variations(tags: Vec<TagType>) -> Vec<Vec<TagType>> {
        let mut variety: Vec<Vec<TagType>> = Vec::new();

        for len in 1..=tags.len() {
            let combination: Vec<Vec<TagType>> = tags
                .iter()
                .cloned() // Clone the TagType instances
                .combinations(len)
                .collect();
            variety.extend(combination);
        }

        variety
    }
    pub fn evaluate(&self, tags: Arc<Mutex<Vec<UiTag>>>) -> Vec<CalcResult> {
        let tags: Vec<TagType> = tags
            .lock()
            .unwrap()
            .iter()
            .map(|f: &UiTag| f.tag_type().clone())
            .collect();
        let variations: Vec<Vec<TagType>> = Self::variations(tags);
        variations
            .iter()
            .filter_map(|variation: &Vec<TagType>| {
                let mut matched_ops: Vec<&Operator> = Vec::new();
                for op in self.pool.iter().clone() {
                    if (self.ignore_tier_1 && op.rarity() == &Rarity::Tier1)
                        && (self.ignore_tier_2 && op.rarity() == &Rarity::Tier2)
                        && (self.ignore_tier_3 && op.rarity() == &Rarity::Tier3)
                    {
                        continue;
                    }
                    if op.matches_tags(variation) {
                        matched_ops.push(op);
                    }
                }

                if matched_ops.len() == 0 {
                    return None;
                }

                Some(CalcResult {
                    tag_variation: variation.clone(),
                    obtainable_operators: matched_ops,
                })
            })
            .collect()
    }
}

#[test]
fn match_combos() -> Result<(), Box<dyn std::error::Error>> {
    use crate::types::operator;
    use crate::types::tag::TagType;
    use itertools::Itertools;

    let mut result = Vec::new();
    let vec = vec![TagType::Ranged, TagType::Survival];
    for len in 1..=vec.len() {
        result.extend(vec.iter().cloned().combinations(len));
    }

    for comb in result {
        for op in operator::load_operator_data()? {
            if op.matches_tags(&comb) {
                println!("{:?} | {:?}", comb, op);
            }
        }
    }

    Ok(())
}
