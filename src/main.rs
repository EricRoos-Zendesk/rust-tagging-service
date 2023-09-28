mod utils;
use std::collections::HashSet;

fn main() {
    let diffs = utils::get_diffs_from(&HashSet::from(["A".to_string()]), &HashSet::from(["B".to_string()]), 0);
    println!("{:?}", diffs);
}

