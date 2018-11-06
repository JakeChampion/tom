use tom::{SyntaxNodeRef, TextUnit, TextRange};

pub fn find_leaf_at_offset(node: SyntaxNodeRef, offset: TextUnit) -> LeafAtOffset {
    let range = node.range();
    assert!(
        contains_offset_nonstrict(range, offset),
        "Bad offset: range {:?} offset {:?}", range, offset
    );
    if range.is_empty() {
        return LeafAtOffset::None;
    }

    if node.is_leaf() {
        return LeafAtOffset::Single(node);
    }

    let mut children = node.children()
        .filter(|child| {
            let child_range = child.range();
            !child_range.is_empty() && contains_offset_nonstrict(child_range, offset)
        });

    let left = children.next().unwrap();
    let right = children.next();
    assert!(children.next().is_none());
    return if let Some(right) = right {
        match (find_leaf_at_offset(left, offset), find_leaf_at_offset(right, offset)) {
            (LeafAtOffset::Single(left), LeafAtOffset::Single(right)) =>
                LeafAtOffset::Between(left, right),
            _ => unreachable!()
        }
    } else {
        find_leaf_at_offset(left, offset)
    };
}

#[derive(Clone, Copy, Debug)]
pub enum LeafAtOffset<'a> {
    None,
    Single(SyntaxNodeRef<'a>),
    Between(SyntaxNodeRef<'a>, SyntaxNodeRef<'a>)
}

impl<'a> LeafAtOffset<'a> {
    pub fn right_biased(self) -> Option<SyntaxNodeRef<'a>> {
        match self {
            LeafAtOffset::None => None,
            LeafAtOffset::Single(node) => Some(node),
            LeafAtOffset::Between(_, right) => Some(right)
        }
    }

    pub fn left_biased(self) -> Option<SyntaxNodeRef<'a>> {
        match self {
            LeafAtOffset::None => None,
            LeafAtOffset::Single(node) => Some(node),
            LeafAtOffset::Between(left, _) => Some(left)
        }
    }
}

impl<'f> Iterator for LeafAtOffset<'f> {
    type Item = SyntaxNodeRef<'f>;

    fn next(&mut self) -> Option<SyntaxNodeRef<'f>> {
        match *self {
            LeafAtOffset::None => None,
            LeafAtOffset::Single(node) => { *self = LeafAtOffset::None; Some(node) }
            LeafAtOffset::Between(left, right) => { *self = LeafAtOffset::Single(right); Some(left) }
        }
    }
}

pub fn find_covering_node(root: SyntaxNodeRef, range: TextRange) -> SyntaxNodeRef {
    assert!(range.is_subrange(&root.range()));
    let (left, right) = match (
        find_leaf_at_offset(root, range.start()).right_biased(),
        find_leaf_at_offset(root, range.end()).left_biased()
    ) {
        (Some(l), Some(r)) => (l, r),
        _ => return root
    };

    common_ancestor(left, right)
}


pub fn contains_offset_nonstrict(range: TextRange, offset: TextUnit) -> bool {
    range.start() <= offset && offset <= range.end()
}

fn common_ancestor<'a>(n1: SyntaxNodeRef<'a>, n2: SyntaxNodeRef<'a>) -> SyntaxNodeRef<'a> {
    for p in n1.ancestors() {
        if n2.ancestors().any(|a| a == p) {
            return p;
        }
    }
    panic!("Can't find common ancestor of {:?} and {:?}", n1, n2)
}
