use std::{hash::Hash, collections::{HashMap}, sync::Arc};

use super::hasher::BuildMurmur3Hasher;

#[derive(Clone, Debug, Eq)]
pub enum Node {
    Domain {
        name: String,
        blocking: bool,
    },
    Wildcard,
}

impl Node {
    pub fn is_blocking(&self) -> bool {
        match self {
            Node::Domain { name: _, blocking } => *blocking,
            Node::Wildcard => true,
        }
    }
}

impl Hash for Node {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Node::Domain {name, blocking: _} => state.write(name.as_str().as_bytes()),
            Node::Wildcard => state.write_u8('*' as u8),
        }
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Domain { name: l_name, blocking: _l_blocking }, Self::Domain { name: r_name, blocking: _r_blocking }) => l_name == r_name,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl From<String> for Node {
    fn from(value: String) -> Self {
        Self::Domain { name: value, blocking: false }
    }
}

#[derive(Clone, Debug)]
pub struct NodeTree(HashMap<Arc<Node>, NodeTree, BuildMurmur3Hasher>);

impl NodeTree {
    pub fn new() -> Self {
        Self(HashMap::with_hasher(BuildMurmur3Hasher))
    }

    pub fn branch(mut self, mut nodes: Vec<Arc<Node>>) -> Self {
        if let Some(node) = nodes.pop() {
            if let Some((key, value)) = self.0.clone().get_key_value(&node) {
                if !key.is_blocking() && node.is_blocking() {
                    self.0.remove(key);
                    self.0.insert(node, value.clone().branch(nodes));
                } else {
                    self.0.insert(key.clone(), value.clone().branch(nodes));
                }
            } else {
                self.0.insert(node, NodeTree::new().branch(nodes));
            }
        }
        self
    }

    pub fn is_branch_blocking(&self, mut nodes: Vec<String>) -> bool {
        if let Some(node) = nodes.pop() {
            let is_last = nodes.is_empty();
            let node: Node = node.into();
            if is_last {
                if let Some((key, _value)) = self.0.get_key_value(&node) {
                    key.is_blocking()
                } else {
                    false
                }
            } else {
                if self.0.contains_key(&Node::Wildcard) {
                    true   
                } else {
                    if let Some(branch) = self.0.get(&node) {
                        branch.is_branch_blocking(nodes)
                    } else {
                        false
                    }
                }
            }
        } else {
            false
        }
    } 
}