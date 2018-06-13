use std::{
    collections::HashMap,
};

use {TomlDoc, TomlNode};

mod node_change;
mod whitespace;

use self::node_change::{
    Changes, MergedChild, ChildChangeOp,
};
use self::whitespace::{compute_ws, Location, Edge};

#[derive(Debug)]
pub struct Edit<'f> {
    doc: &'f TomlDoc,
    ops: HashMap<TomlNode<'f>, Changes<'f>>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Position<'f> {
    After(TomlNode<'f>),
    Before(TomlNode<'f>),
    StartOf(TomlNode<'f>),
    EndOf(TomlNode<'f>),
}

impl<'f> Position<'f> {
    pub fn after(node: impl Into<TomlNode<'f>>) -> Position<'f> {
        Position::After(node.into())
    }
    pub fn before(node: impl Into<TomlNode<'f>>) -> Position<'f> {
        Position::Before(node.into())
    }
    pub fn start_of(node: impl Into<TomlNode<'f>>) -> Position<'f> {
        Position::StartOf(node.into())
    }
    pub fn end_of(node: impl Into<TomlNode<'f>>) -> Position<'f> {
        Position::EndOf(node.into())
    }
}

impl<'f> Edit<'f> {
    pub fn new(doc: &'f TomlDoc) -> Edit {
        Edit { doc, ops: HashMap::new() }
    }

    pub fn replace(
        &mut self,
        node: impl Into<TomlNode<'f>>,
        replacement: impl Into<TomlNode<'f>>,
    ) {
        let (parent, pos) = parent(node.into());
        self.changes_mut(parent).add_child_change(
            pos, ChildChangeOp::Replace(replacement.into()),
        );
    }

    pub fn insert(
        &mut self,
        node: impl Into<TomlNode<'f>>,
        position: Position<'f>,
    ) {
        let (parent, pos) = match position {
            Position::After(a) => {
                let (parent, position) = parent(a);
                (parent, position + 1)
            }
            Position::Before(a) => {
                let (parent, position) = parent(a);
                (parent, position)
            }
            Position::StartOf(a) => (a, 0),
            Position::EndOf(a) => (a, a.children().count()),
        };
        self.changes_mut(parent)
            .add_child_change(pos, ChildChangeOp::Insert(node.into()));
    }

    pub fn delete(&mut self, node: impl Into<TomlNode<'f>>) {
        let (parent, pos) = parent(node.into());
        self.changes_mut(parent)
            .add_child_change(pos, ChildChangeOp::Delete)
    }

    pub fn finish(self) -> String {
        let root = self.doc.parse_tree();
        self.rendered(root, 0)
    }

    fn changes(&self, target: TomlNode<'f>) -> Option<&Changes<'f>> {
        self.ops.get(&target)
    }

    fn changes_mut(&mut self, target: TomlNode<'f>) -> &mut Changes<'f> {
        self.ops.entry(target).or_insert_with(Default::default)
    }
}

impl<'f> Edit<'f> {
    fn rendered(&self, node: TomlNode<'f>, level: u16) -> String {
        if level > 999 {
            covered_by!("infinite_doc");
            panic!("Infinite edit");
        }
        match self.changes(node) {
            None => {
                if node.is_leaf() {
                    node.text().to_owned()
                } else {
                    let mut buff = String::new();
                    for child in node.children() {
                        buff += &self.rendered(child, level + 1);
                    }
                    buff
                }
            }
            Some(changes) => {
                let mut buff = String::new();
                let mut prev: Option<(bool, TomlNode)> = None;
                for m in changes.merge(node.children()) {
                    match m {
                        MergedChild::Old(child) => {
                            match prev {
                                Some((prev_old, prev)) => {
                                    if !prev_old {
                                        buff += &compute_ws(
                                            Location::Between(prev, child),
                                        )
                                    }
                                }
                                _ => (),
                            };
                            buff += &self.rendered(child, level + 1);
                            prev = Some((true, child));
                        }
                        MergedChild::Deleted(_) => (),
                        MergedChild::Replaced(new_child) => {
                            buff += &self.rendered(new_child, level + 1);
                            prev = Some((false, new_child));
                        }
                        MergedChild::Inserted(new_child) => {
                            buff += &match prev {
                                Some((_, prev)) => compute_ws(
                                    Location::Between(prev, new_child),
                                ),
                                None => compute_ws(
                                    Location::OnEdge { child: new_child, parent: node, edge: Edge::Left }
                                ),
                            };
                            buff += &self.rendered(new_child, level + 1);
                            prev = Some((false, new_child));
                        }
                    }
                }
                match prev {
                    Some((false, new_child)) =>
                        buff += &compute_ws(
                            Location::OnEdge { child: new_child, parent: node, edge: Edge::Right }
                        ),
                    _ => (),
                }
                buff
            }
        }
    }
}

fn parent<'f>(child: TomlNode<'f>) -> (TomlNode<'f>, usize) {
    let parent = child.parent().unwrap();
    let position = parent.children().position(|it| it == child).unwrap();
    (parent, position)
}
