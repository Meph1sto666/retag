
#[test]
fn match_combos() -> Result<(), Box<dyn std::error::Error>> {
    use crate::types::operator;
    use crate::types::tag::TagType;
    use itertools::Itertools;

    let mut result = Vec::new();
    let vec = vec![TagType::Ranged, TagType::Survival];
    for len in 1..=vec.len() {
        let combos = vec.iter().combinations(len);
        result.extend(combos);
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
