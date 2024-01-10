use std::collections::HashMap;
use std::fmt::Display;

pub(crate) trait Node<Id> {
    fn root() -> Id;

    fn id(&self) -> Id;

    fn parent(&self) -> Option<Id>;
}

#[derive(Debug)]
pub(crate) struct Tree<Id, Node> {
    nodes: HashMap<Id, Node>,
    children: HashMap<Id, Vec<Node>>,
}

impl<Id, Node> Tree<Id, Node>
where
    Id: Eq + std::hash::Hash + Ord,
    Node: crate::tree::Node<Id> + Display + Clone,
{
    pub(crate) fn new(nodes: impl Iterator<Item = Node>) -> Self {
        let mut map = HashMap::new();
        let mut children = HashMap::new();
        for node in nodes {
            map.insert(node.id(), node.clone());
            if let Some(parent) = node.parent() {
                children.entry(parent).or_insert(Vec::new()).push(node);
            }
        }
        for (_, children) in children.iter_mut() {
            children.sort_by_key(|a| a.id());
        }
        Tree {
            nodes: map,
            children,
        }
    }

    fn root(&self) -> &Node {
        self.nodes
            .get(&Node::root())
            .ok_or_else(|| todo!())
            .unwrap()
    }

    fn children(&self, node: &Node) -> Vec<Node> {
        self.children.get(&node.id()).cloned().unwrap_or(Vec::new())
    }

    pub(crate) fn format(&self) -> String {
        let mut acc = "".to_string();
        self.format_helper(self.root(), true, true, &mut Vec::new(), &mut acc);
        acc.to_string()
    }

    fn format_helper(
        &self,
        node: &Node,
        is_root: bool,
        is_last: bool,
        prefixes: &mut Vec<&str>,
        acc: &mut String,
    ) {
        for prefix in prefixes.iter() {
            *acc += prefix;
        }
        if !is_root {
            let has_children = !self.children(node).is_empty();
            *acc += if is_last { "└─" } else { "├─" };
            *acc += if has_children { "┬ " } else { "─ " };
        }
        *acc += &node.to_string();
        *acc += "\n";
        let children = self.children(node);
        for (i, child) in children.iter().enumerate() {
            if !is_root {
                prefixes.push(if is_last { "  " } else { "│ " });
            }
            let is_last = i == children.len() - 1;
            self.format_helper(child, false, is_last, prefixes, acc);
            prefixes.pop();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use unindent::Unindent;

    #[derive(Clone)]
    struct TestNode {
        id: u8,
        name: String,
        parent: Option<u8>,
    }

    impl Display for TestNode {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.name)
        }
    }

    impl Node<u8> for TestNode {
        fn root() -> u8 {
            1
        }

        fn id(&self) -> u8 {
            self.id
        }

        fn parent(&self) -> Option<u8> {
            self.parent
        }
    }

    impl TestNode {
        fn new(id: u8, name: &str, parent: Option<u8>) -> TestNode {
            TestNode {
                id,
                name: name.to_string(),
                parent,
            }
        }
    }

    #[test]
    fn a_single_node_tree() {
        let tree = Tree::new(vec![TestNode::new(1, "test-node", None)].into_iter());
        assert_eq!(
            tree.format(),
            "
              test-node
            "
            .unindent()
        );
    }

    #[test]
    fn b_child() {
        let tree = Tree::new(
            vec![
                TestNode::new(1, "parent", None),
                TestNode::new(2, "child", Some(1)),
            ]
            .into_iter(),
        );
        assert_eq!(
            tree.format(),
            "
              parent
              └── child
            "
            .unindent()
        );
    }

    #[test]
    fn c_children() {
        let tree = Tree::new(
            vec![
                TestNode::new(1, "parent", None),
                TestNode::new(2, "foo", Some(1)),
                TestNode::new(3, "bar", Some(1)),
                TestNode::new(4, "baz", Some(1)),
            ]
            .into_iter(),
        );
        assert_eq!(
            tree.format(),
            "
              parent
              ├── foo
              ├── bar
              └── baz
            "
            .unindent()
        );
    }

    #[test]
    fn d_grandchildren() {
        let tree = Tree::new(
            vec![
                TestNode::new(1, "parent", None),
                TestNode::new(2, "foo", Some(1)),
                TestNode::new(3, "bar", Some(2)),
            ]
            .into_iter(),
        );
        assert_eq!(
            tree.format(),
            "
              parent
              └─┬ foo
                └── bar
            "
            .unindent()
        );
    }

    #[test]
    fn e_bigger() {
        let tree = Tree::new(
            vec![
                TestNode::new(1, "parent", None),
                TestNode::new(2, "foo", Some(1)),
                TestNode::new(3, "bar", Some(2)),
                TestNode::new(4, "baz", Some(1)),
            ]
            .into_iter(),
        );
        assert_eq!(
            tree.format(),
            "
              parent
              ├─┬ foo
              │ └── bar
              └── baz
            "
            .unindent()
        );
    }
}
