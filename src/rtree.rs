use std::{
    fmt,
    hash::{Hash, Hasher},
};

use rowan::{Types, SmolStr, TextRange};

use ::{Symbol, SyntaxError};


pub use rowan::TreeRoot;

#[derive(Debug, Clone, Copy)]
pub enum TomTypes {}
impl Types for TomTypes {
    type Kind = Symbol;
    type RootData = Vec<SyntaxError>;
}

pub type OwnedRoot = ::rowan::OwnedRoot<TomTypes>;
pub type RefRoot<'a> = ::rowan::RefRoot<'a, TomTypes>;

pub type GreenNode = ::rowan::GreenNode<TomTypes>;

#[derive(Clone, Copy)]
pub struct SyntaxNode<R: TreeRoot<TomTypes> = OwnedRoot>(::rowan::SyntaxNode<TomTypes, R>);
pub type SyntaxNodeRef<'a> = SyntaxNode<RefRoot<'a>>;

impl<R1, R2> PartialEq<SyntaxNode<R1>> for SyntaxNode<R2>
where
    R1: TreeRoot<TomTypes>,
    R2: TreeRoot<TomTypes>,
{
    fn eq(&self, other: &SyntaxNode<R1>) -> bool {
        self.0 == other.0
    }
}

impl<R: TreeRoot<TomTypes>> Eq for SyntaxNode<R> {}
impl<R: TreeRoot<TomTypes>> Hash for SyntaxNode<R> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl SyntaxNode {
    pub(crate) fn new(green: GreenNode, errors: Vec<SyntaxError>) -> SyntaxNode {
        SyntaxNode(::rowan::SyntaxNode::new(green, errors))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Next,
    Prev,
}

impl<'a> SyntaxNodeRef<'a> {
    pub fn leaf_text(self) -> Option<&'a SmolStr> {
        self.0.leaf_text()
    }
    // pub fn ancestors(self) -> impl Iterator<Item = SyntaxNodeRef<'a>> {
    //     crate::algo::generate(Some(self), |&node| node.parent())
    // }

    // pub fn descendants(self) -> impl Iterator<Item = SyntaxNodeRef<'a>> {
    //     crate::algo::walk::walk(self).filter_map(|event| match event {
    //         crate::algo::walk::WalkEvent::Enter(node) => Some(node),
    //         crate::algo::walk::WalkEvent::Exit(_) => None,
    //     })
    // }

    // pub fn siblings(self, direction: Direction) -> impl Iterator<Item = SyntaxNodeRef<'a>> {
    //     crate::algo::generate(Some(self), move |&node| match direction {
    //         Direction::Next => node.next_sibling(),
    //         Direction::Prev => node.prev_sibling(),
    //     })
    // }
}

impl<R: TreeRoot<TomTypes>> SyntaxNode<R> {
//     pub(crate) fn root_data(&self) -> &Vec<SyntaxError> {
//         self.0.root_data()
//     }
    pub(crate) fn replace_with(&self, replacement: GreenNode) -> GreenNode {
        self.0.replace_with(replacement)
    }
    pub fn borrowed<'a>(&'a self) -> SyntaxNode<RefRoot<'a>> {
        SyntaxNode(self.0.borrowed())
    }
    pub fn owned(&self) -> SyntaxNode<OwnedRoot> {
        SyntaxNode(self.0.owned())
    }
    pub fn symbol(&self) -> Symbol {
        self.0.kind()
    }
    pub fn range(&self) -> TextRange {
        self.0.range()
    }
//     pub fn text(&self) -> SyntaxText {
//         SyntaxText::new(self.borrowed())
//     }
    pub fn is_leaf(&self) -> bool {
        self.0.is_leaf()
    }
    pub fn parent(&self) -> Option<SyntaxNode<R>> {
        self.0.parent().map(SyntaxNode)
    }
    pub fn first_child(&self) -> Option<SyntaxNode<R>> {
        self.0.first_child().map(SyntaxNode)
    }
    pub fn last_child(&self) -> Option<SyntaxNode<R>> {
        self.0.last_child().map(SyntaxNode)
    }
    pub fn next_sibling(&self) -> Option<SyntaxNode<R>> {
        self.0.next_sibling().map(SyntaxNode)
    }
    pub fn prev_sibling(&self) -> Option<SyntaxNode<R>> {
        self.0.prev_sibling().map(SyntaxNode)
    }
    pub fn children(&self) -> SyntaxNodeChildren<R> {
        SyntaxNodeChildren(self.0.children())
    }
}

impl<R: TreeRoot<TomTypes>> fmt::Debug for SyntaxNode<R> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:?}@{:?}", self.symbol(), self.range())?;
        // if has_short_text(self.kind()) {
        //     write!(fmt, " \"{}\"", self.text())?;
        // }
        Ok(())
    }
}

#[derive(Debug)]
pub struct SyntaxNodeChildren<R: TreeRoot<TomTypes>>(::rowan::SyntaxNodeChildren<TomTypes, R>);

impl<R: TreeRoot<TomTypes>> Iterator for SyntaxNodeChildren<R> {
    type Item = SyntaxNode<R>;

    fn next(&mut self) -> Option<SyntaxNode<R>> {
        self.0.next().map(SyntaxNode)
    }
}
