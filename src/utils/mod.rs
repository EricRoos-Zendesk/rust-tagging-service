use std::collections::HashSet;
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum TaggingOperation {
    Add(String),
    Remove(String)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TaggingDelta {
    pub timestamp_epoch_ms: u128,
    pub operation: TaggingOperation,
    pub account_id: i64,
    pub ticket_id: i64
}

pub fn get_diffs_from(original_tags: &HashSet<String>, next_state: &HashSet<String>, timestamp: u128, ticket_id: i64) -> Vec<TaggingDelta> {
    let added = next_state.difference(original_tags);
    let removed = original_tags.difference(next_state);
    let mut diffs = Vec::new();
    for tag in added.into_iter() {
        diffs.push(TaggingDelta {
            timestamp_epoch_ms: timestamp,
            operation: TaggingOperation::Add(tag.to_string()),
            account_id: 0,
            ticket_id: ticket_id
        });
    }
    for tag in removed.into_iter() {
        diffs.push(TaggingDelta {
            timestamp_epoch_ms: timestamp,
            operation: TaggingOperation::Remove(tag.to_string()),
            account_id: 0,
            ticket_id: 0
        });
    }
    diffs
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use crate::utils;

    #[test]
    fn next_state_adds_one() {
        let mut original = HashSet::new();
        original.insert("A".to_string());
        original.insert("B".to_string());

        let mut next_state = HashSet::new();
        next_state.insert("A".to_string());
        next_state.insert("B".to_string());
        next_state.insert("C".to_string());

        let result = utils::get_diffs_from(&original, &next_state, 0,0);
        let mut expected = Vec::new();
        expected.push(utils::TaggingDelta {
            timestamp_epoch_ms: 0,
            operation: utils::TaggingOperation::Add("C".to_string()),
            account_id: 0,
            ticket_id: 0
        });
        assert_eq!(result, expected);
    }

    #[test]
    fn next_state_removes_one() {
        let mut original = HashSet::new();
        original.insert("A".to_string());
        original.insert("B".to_string());

        let mut next_state = HashSet::new();
        next_state.insert("A".to_string());

        let result = utils::get_diffs_from(&original, &next_state, 0,0);
        let mut expected = Vec::new();
        expected.push(utils::TaggingDelta {
            timestamp_epoch_ms: 0,
            operation: utils::TaggingOperation::Remove("B".to_string()),
            account_id: 0,
            ticket_id: 0
        });
        assert_eq!(result, expected);
    }
}
