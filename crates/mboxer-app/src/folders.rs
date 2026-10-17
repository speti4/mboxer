//! The folder pane's label order.

use crate::types::LabelCount;

/// Gmail's system labels, in the order a mail client shows them; everything
/// else follows alphabetically.
const WELL_KNOWN: [&str; 10] = [
    "Inbox",
    "Starred",
    "Important",
    "Sent",
    "Drafts",
    "Spam",
    "Trash",
    "Archived",
    "Unread",
    "Opened",
];

/// Orders `counts` (label, messages) for display.
pub fn ordered_labels(counts: Vec<(String, u64)>) -> Vec<LabelCount> {
    let mut labels: Vec<LabelCount> = counts
        .into_iter()
        .map(|(name, count)| LabelCount { name, count })
        .collect();
    labels.sort_by_cached_key(|label| {
        let rank = WELL_KNOWN
            .iter()
            .position(|known| known.eq_ignore_ascii_case(&label.name))
            .unwrap_or(WELL_KNOWN.len());
        (rank, label.name.to_lowercase())
    });
    labels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_labels_first_then_alphabetical() {
        let counts = vec![
            ("Zebra".to_owned(), 1),
            ("Category personal".to_owned(), 5),
            ("Sent".to_owned(), 2),
            ("alpha".to_owned(), 3),
            ("Inbox".to_owned(), 9),
            ("Opened".to_owned(), 4),
        ];
        let names: Vec<_> = ordered_labels(counts)
            .into_iter()
            .map(|label| label.name)
            .collect();
        assert_eq!(
            names,
            [
                "Inbox",
                "Sent",
                "Opened",
                "alpha",
                "Category personal",
                "Zebra"
            ]
        );
    }
}
