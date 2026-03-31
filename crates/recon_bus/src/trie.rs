//! Arena-based topic trie for wildcard subscription matching.

use std::{collections::HashMap, sync::Arc};

use crate::topic::Topic;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct SubscriberId(pub u64);

type NodeId = usize;

#[derive(Debug, Default)]
struct TrieNode {
    subscribers: Vec<SubscriberId>,
    children: HashMap<Arc<str>, NodeId>,
    single_wildcard: Option<NodeId>,
    multi_wildcard_subscribers: Vec<SubscriberId>,
}

/// Arena-based topic trie for wildcard matching.
///
/// Nodes are stored in a flat `Vec` indexed by `NodeId`. All operations
/// are iterative.
#[derive(Debug)]
pub(crate) struct TopicTrie {
    nodes: Vec<TrieNode>,
}

impl TopicTrie {
    pub fn new() -> Self {
        Self {
            nodes: vec![TrieNode::default()],
        }
    }

    pub fn insert(&mut self, filter: &Topic, id: SubscriberId) {
        let mut current = 0;
        let count = filter.segment_count();

        for i in 0..count {
            let seg = filter.segment(i);
            match seg {
                "**" => {
                    self.nodes[current].multi_wildcard_subscribers.push(id);
                    return;
                }
                "*" => {
                    let child = if let Some(child_id) = self.nodes[current].single_wildcard {
                        child_id
                    } else {
                        let child_id = self.alloc_node();
                        self.nodes[current].single_wildcard = Some(child_id);
                        child_id
                    };

                    if i == count - 1 {
                        self.nodes[child].subscribers.push(id);
                        return;
                    }
                    current = child;
                }
                literal => {
                    let key: Arc<str> = Arc::from(literal);
                    let child = if let Some(&child_id) = self.nodes[current].children.get(&key) {
                        child_id
                    } else {
                        let child_id = self.alloc_node();
                        self.nodes[current].children.insert(key, child_id);
                        child_id
                    };

                    if i == count - 1 {
                        self.nodes[child].subscribers.push(id);
                        return;
                    }
                    current = child;
                }
            }
        }
    }

    pub fn remove(&mut self, filter: &Topic, id: SubscriberId) {
        let mut current = 0;
        let count = filter.segment_count();

        for i in 0..count {
            let seg = filter.segment(i);
            match seg {
                "**" => {
                    self.nodes[current]
                        .multi_wildcard_subscribers
                        .retain(|s| *s != id);
                    return;
                }
                "*" => {
                    let Some(child) = self.nodes[current].single_wildcard else {
                        return;
                    };
                    if i == count - 1 {
                        self.nodes[child].subscribers.retain(|s| *s != id);
                        return;
                    }
                    current = child;
                }
                literal => {
                    let Some(&child) = self.nodes[current].children.get(literal) else {
                        return;
                    };
                    if i == count - 1 {
                        self.nodes[child].subscribers.retain(|s| *s != id);
                        return;
                    }
                    current = child;
                }
            }
        }
    }

    /// Find all subscriber IDs matching a concrete topic.
    pub fn matching(&self, topic: &Topic) -> Vec<SubscriberId> {
        let mut result = Vec::new();
        let seg_count = topic.segment_count();
        let mut stack: Vec<(NodeId, usize)> = vec![(0, 0)];

        while let Some((node_id, seg_idx)) = stack.pop() {
            let node = &self.nodes[node_id];

            node.multi_wildcard_subscribers
                .iter()
                .for_each(|id| result.push(*id));

            if seg_idx == seg_count {
                node.subscribers.iter().for_each(|id| result.push(*id));
                continue;
            }

            let seg = topic.segment(seg_idx);

            if let Some(child_id) = node.single_wildcard {
                stack.push((child_id, seg_idx + 1));
            }

            if let Some(&child_id) = node.children.get(seg) {
                stack.push((child_id, seg_idx + 1));
            }
        }

        result
    }

    fn alloc_node(&mut self) -> NodeId {
        let id = self.nodes.len();
        self.nodes.push(TrieNode::default());
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn topic(s: &str) -> Topic {
        Topic::try_from(s).unwrap()
    }

    #[test]
    fn exact_match() {
        let mut trie = TopicTrie::new();
        trie.insert(&topic("game/valorant/status"), SubscriberId(1));

        assert_eq!(
            trie.matching(&topic("game/valorant/status")),
            vec![SubscriberId(1)]
        );
        assert!(trie.matching(&topic("game/apex/status")).is_empty());
    }

    #[test]
    fn single_wildcard() {
        let mut trie = TopicTrie::new();
        trie.insert(&topic("game/*/status"), SubscriberId(1));

        assert_eq!(
            trie.matching(&topic("game/valorant/status")),
            vec![SubscriberId(1)]
        );
        assert_eq!(
            trie.matching(&topic("game/apex/status")),
            vec![SubscriberId(1)]
        );
        assert!(trie.matching(&topic("game/valorant/health")).is_empty());
        assert!(trie.matching(&topic("game/status")).is_empty());
    }

    #[test]
    fn multi_wildcard() {
        let mut trie = TopicTrie::new();
        trie.insert(&topic("game/**"), SubscriberId(1));

        assert_eq!(
            trie.matching(&topic("game/valorant")),
            vec![SubscriberId(1)]
        );
        assert_eq!(
            trie.matching(&topic("game/valorant/status")),
            vec![SubscriberId(1)]
        );
        assert_eq!(trie.matching(&topic("game/a/b/c")), vec![SubscriberId(1)]);
        assert!(trie.matching(&topic("other/thing")).is_empty());
    }

    #[test]
    fn multi_wildcard_matches_parent_level() {
        let mut trie = TopicTrie::new();
        trie.insert(&topic("game/**"), SubscriberId(1));

        assert_eq!(trie.matching(&topic("game")), vec![SubscriberId(1)]);
    }

    #[test]
    fn root_double_star() {
        let mut trie = TopicTrie::new();
        trie.insert(&topic("**"), SubscriberId(1));

        assert_eq!(trie.matching(&topic("anything")), vec![SubscriberId(1)]);
        assert_eq!(trie.matching(&topic("a/b/c")), vec![SubscriberId(1)]);
    }

    #[test]
    fn multiple_subscribers() {
        let mut trie = TopicTrie::new();
        trie.insert(&topic("game/*/status"), SubscriberId(1));
        trie.insert(&topic("game/valorant/status"), SubscriberId(2));
        trie.insert(&topic("**"), SubscriberId(3));

        let mut result = trie.matching(&topic("game/valorant/status"));
        result.sort_by_key(|s| s.0);
        assert_eq!(
            result,
            vec![SubscriberId(1), SubscriberId(2), SubscriberId(3)]
        );
    }

    #[test]
    fn remove_subscriber() {
        let mut trie = TopicTrie::new();
        trie.insert(&topic("game/*/status"), SubscriberId(1));
        trie.insert(&topic("game/*/status"), SubscriberId(2));

        trie.remove(&topic("game/*/status"), SubscriberId(1));

        assert_eq!(
            trie.matching(&topic("game/valorant/status")),
            vec![SubscriberId(2)]
        );
    }

    #[test]
    fn remove_multi_wildcard() {
        let mut trie = TopicTrie::new();
        trie.insert(&topic("game/**"), SubscriberId(1));
        trie.remove(&topic("game/**"), SubscriberId(1));

        assert!(trie.matching(&topic("game/valorant")).is_empty());
    }

    #[test]
    fn remove_nonexistent_is_noop() {
        let mut trie = TopicTrie::new();
        trie.insert(&topic("a/b"), SubscriberId(1));
        trie.remove(&topic("x/y"), SubscriberId(1));
        trie.remove(&topic("a/b"), SubscriberId(99));

        assert_eq!(trie.matching(&topic("a/b")), vec![SubscriberId(1)]);
    }
}
