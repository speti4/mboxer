//! The folder pane's label tree and its order.

use crate::types::LabelNode;

/// Gmail's system labels, in the order a mail client shows them; everything
/// else follows alphabetically. `Bin` is `Trash` in a UK-English Takeout.
const WELL_KNOWN: [&str; 12] = [
    "Inbox",
    "Starred",
    "Important",
    "Sent",
    "Drafts",
    "Spam",
    "Trash",
    "Bin",
    "Archived",
    "Unread",
    "Opened",
    "Chat",
];

/// The synthetic node that groups the `Category <name>` labels of a Takeout.
const CATEGORIES: &str = "Categories";
const CATEGORY_PREFIX: &str = "category ";

/// A node under construction: the label when messages carry it, its count,
/// and the children by their path segment, in insertion order.
#[derive(Default)]
struct Draft {
    label: Option<String>,
    count: u64,
    children: Vec<(String, Draft)>,
}

impl Draft {
    fn child(&mut self, name: &str) -> &mut Draft {
        if let Some(i) = self.children.iter().position(|(n, _)| n == name) {
            return &mut self.children[i].1;
        }
        self.children.push((name.to_owned(), Draft::default()));
        &mut self.children.last_mut().expect("just pushed").1
    }

    fn into_nodes(self) -> Vec<LabelNode> {
        let mut nodes: Vec<LabelNode> = self
            .children
            .into_iter()
            .map(|(name, draft)| {
                let Draft {
                    label,
                    count,
                    children,
                } = draft;
                let children = Draft {
                    children,
                    ..Draft::default()
                }
                .into_nodes();
                let total = count + children.iter().map(|c| c.total).sum::<u64>();
                let labels = label
                    .iter()
                    .cloned()
                    .chain(children.iter().flat_map(|c| c.labels.iter().cloned()))
                    .collect();
                LabelNode {
                    name,
                    label,
                    count,
                    total,
                    labels,
                    children,
                }
            })
            .collect();
        nodes.sort_by_cached_key(|node| (rank(&node.name), node.name.to_lowercase()));
        nodes
    }
}

/// Display rank of a node name: system labels first in their order, then the
/// categories node, then everything else (alphabetical through the sort key).
fn rank(name: &str) -> usize {
    WELL_KNOWN
        .iter()
        .position(|known| known.eq_ignore_ascii_case(name))
        .unwrap_or(if name == CATEGORIES {
            WELL_KNOWN.len()
        } else {
            WELL_KNOWN.len() + 1
        })
}

/// The path of a label in the tree: `Work/Invoices` → `[Work, Invoices]`,
/// `Category updates` → `[Categories, Updates]`, anything else one segment.
fn segments(label: &str) -> Vec<String> {
    if label.len() > CATEGORY_PREFIX.len()
        && label[..CATEGORY_PREFIX.len()].eq_ignore_ascii_case(CATEGORY_PREFIX)
    {
        let rest = label[CATEGORY_PREFIX.len()..].trim();
        if !rest.is_empty() {
            let mut chars = rest.chars();
            let name = match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => rest.to_owned(),
            };
            return vec![CATEGORIES.to_owned(), name];
        }
    }
    let parts: Vec<String> = label
        .split('/')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_owned)
        .collect();
    if parts.is_empty() {
        vec![label.to_owned()]
    } else {
        parts
    }
}

/// Builds the label tree of `counts` (label, messages) for display.
pub fn label_tree(counts: Vec<(String, u64)>) -> Vec<LabelNode> {
    let mut root = Draft::default();
    for (label, count) in counts {
        let mut node = &mut root;
        for segment in segments(&label) {
            node = node.child(&segment);
        }
        node.label = Some(label);
        node.count += count;
    }
    root.into_nodes()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree(counts: &[(&str, u64)]) -> Vec<LabelNode> {
        label_tree(
            counts
                .iter()
                .map(|(name, count)| ((*name).to_owned(), *count))
                .collect(),
        )
    }

    fn names(nodes: &[LabelNode]) -> Vec<&str> {
        nodes.iter().map(|node| node.name.as_str()).collect()
    }

    #[test]
    fn system_labels_first_then_categories_then_alphabetical() {
        let nodes = tree(&[
            ("Zebra", 1),
            ("Category personal", 5),
            ("Sent", 2),
            ("alpha", 3),
            ("Inbox", 9),
            ("Bin", 1),
            ("Opened", 4),
        ]);
        assert_eq!(
            names(&nodes),
            [
                "Inbox",
                "Sent",
                "Bin",
                "Opened",
                "Categories",
                "alpha",
                "Zebra"
            ]
        );
        assert!(nodes.iter().all(|node| node.labels.len() == 1));
        assert_eq!(nodes[0].total, 9);
    }

    #[test]
    fn slashes_build_a_hierarchy_and_a_parent_covers_its_children() {
        let nodes = tree(&[
            ("Work", 2),
            ("Work/Invoices", 1),
            ("Work/Invoices/2026", 1),
            ("Workshop", 1),
            ("Work-archive", 1),
        ]);
        assert_eq!(names(&nodes), ["Work", "Work-archive", "Workshop"]);
        let work = &nodes[0];
        assert_eq!(work.label.as_deref(), Some("Work"));
        assert_eq!((work.count, work.total), (2, 4));
        assert_eq!(work.labels, ["Work", "Work/Invoices", "Work/Invoices/2026"]);
        assert_eq!(names(&work.children), ["Invoices"]);
        let invoices = &work.children[0];
        assert_eq!(invoices.label.as_deref(), Some("Work/Invoices"));
        assert_eq!((invoices.count, invoices.total), (1, 2));
        assert_eq!(invoices.labels, ["Work/Invoices", "Work/Invoices/2026"]);
        assert_eq!(names(&invoices.children), ["2026"]);
        assert_eq!(invoices.children[0].labels, ["Work/Invoices/2026"]);
        assert!(invoices.children[0].children.is_empty());
    }

    #[test]
    fn a_parent_without_messages_is_a_container() {
        let nodes = tree(&[("Work/Invoices", 3), ("Inbox", 1)]);
        assert_eq!(names(&nodes), ["Inbox", "Work"]);
        let work = &nodes[1];
        assert_eq!(work.label, None);
        assert_eq!((work.count, work.total), (0, 3));
        assert_eq!(work.labels, ["Work/Invoices"]);
        assert_eq!(work.children[0].count, 3);
    }

    #[test]
    fn takeout_categories_gather_under_one_node() {
        let nodes = tree(&[
            ("Category updates", 4),
            ("Category personal", 6),
            ("CATEGORY forums", 1),
            ("Category", 2),
            ("Inbox", 1),
        ]);
        assert_eq!(names(&nodes), ["Inbox", "Categories", "Category"]);
        let categories = &nodes[1];
        assert_eq!(categories.label, None);
        assert_eq!((categories.count, categories.total), (0, 11));
        assert_eq!(
            names(&categories.children),
            ["Forums", "Personal", "Updates"]
        );
        assert_eq!(
            categories.labels,
            ["CATEGORY forums", "Category personal", "Category updates"]
        );
        assert_eq!(
            categories.children[1].label.as_deref(),
            Some("Category personal")
        );
    }

    #[test]
    fn odd_labels_stay_one_node() {
        let nodes = tree(&[("/", 1), ("a//b", 1), ("/lead", 1)]);
        assert_eq!(names(&nodes), ["/", "a", "lead"]);
        assert_eq!(nodes[1].children[0].label.as_deref(), Some("a//b"));
        assert_eq!(nodes[2].label.as_deref(), Some("/lead"));
    }
}
