use std::{cmp::Ordering, fmt::Display, rc::Rc};

const fn fibonacci_sequence<const N: usize>() -> [usize; N] {
    let mut seq = [0; N];

    let mut i = 1;
    while i < N {
        if i < 3 {
            seq[i] = 1;
        } else {
            seq[i] = seq[i - 1] + seq[i - 2];
        }

        i += 1;
    }

    seq
}

const FIBONACCI_SEQUENCE: [usize; 94] = fibonacci_sequence::<94>();

#[derive(Debug, Clone, PartialEq)]
pub enum Rope {
    Leaf(Rc<str>),
    Node {
        left: Option<Rc<Self>>,
        right: Option<Rc<Self>>,
        depth: usize,
        weight: usize,
        len: usize,
    },
}

impl Default for Rope {
    fn default() -> Self {
        Self::Leaf(Rc::from(""))
    }
}

impl Display for Rope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.leaves_str().try_for_each(|s| f.write_str(s))
    }
}

impl Rope {
    fn filter_empty(node: Option<Self>) -> Option<Self> {
        match node {
            Some(Self::Leaf(s)) if s.is_empty() => None,
            other => other,
        }
    }

    pub fn leaf(s: &str) -> Self {
        Self::Leaf(Rc::from(s))
    }

    fn node(left: Option<Self>, right: Option<Self>) -> Self {
        let left_res = Self::filter_empty(left);
        let right_res = Self::filter_empty(right);

        assert!(
            left_res.is_some() || right_res.is_some(),
            "Can't have an internal node with empty children. Use Rope::default or Rope::leaf"
        );

        let left_depth = left_res.as_ref().map_or(0, Self::depth);
        let right_depth = right_res.as_ref().map_or(0, Self::depth);

        let left_len = left_res.as_ref().map_or(0, Self::len);
        let right_len = right_res.as_ref().map_or(0, Self::len);

        Self::Node {
            depth: 1 + left_depth.max(right_depth),
            weight: left_len,
            len: left_len + right_len,
            left: left_res.map(Rc::from),
            right: right_res.map(Rc::from),
        }
    }

    fn get_left(&self) -> Option<&Self> {
        match self {
            Self::Leaf(_) => None,
            Self::Node { left, .. } => left.as_deref(),
        }
    }

    fn get_right(&self) -> Option<&Self> {
        match self {
            Self::Leaf(_) => None,
            Self::Node { right, .. } => right.as_deref(),
        }
    }

    fn depth(&self) -> usize {
        match self {
            Self::Leaf(_) => 0,
            Self::Node { depth, .. } => *depth,
        }
    }

    fn weight(&self) -> usize {
        match self {
            Self::Leaf(s) => s.len(),
            Self::Node { weight, .. } => *weight,
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::Leaf(s) => s.len(),
            Self::Node { len, .. } => *len,
        }
    }

    fn is_leaf(&self) -> bool {
        matches!(self, Self::Leaf(_))
    }

    fn is_node(&self) -> bool {
        matches!(self, Self::Node { .. })
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn is_balanced(&self) -> bool {
        let depth = self.depth();
        let len = self.len();

        if depth + 2 >= FIBONACCI_SEQUENCE.len() {
            return false;
        }

        FIBONACCI_SEQUENCE[depth + 2] <= len
    }

    fn leaves(&self) -> impl Iterator<Item = &Self> {
        let mut stack = vec![];
        let mut c = Some(self);

        while let Some(node) = c {
            stack.push(node);
            c = node.get_left();
        }

        std::iter::from_fn(move || {
            while let Some(node) = stack.pop() {
                if node.is_leaf() {
                    return Some(node);
                }

                let mut curr = node.get_right();
                while let Some(child) = curr {
                    stack.push(child);
                    curr = child.get_left();
                }
            }

            None
        })
    }

    fn leaves_str(&self) -> impl Iterator<Item = &str> {
        self.leaves().map(|l| match l {
            Self::Leaf(s) => &**s,
            _ => unreachable!("Rope::leaves yields only leaves"),
        })
    }

    fn into_leaves(self) -> impl Iterator<Item = Self> {
        let mut stack = vec![self];

        std::iter::from_fn(move || {
            while let Some(node) = stack.pop() {
                match node {
                    Self::Leaf(_) => {
                        return Some(node);
                    }
                    Self::Node { left, right, .. } => {
                        if let Some(r) = right.map(Rc::unwrap_or_clone) {
                            stack.push(r);
                        }
                        if let Some(l) = left.map(Rc::unwrap_or_clone) {
                            stack.push(l);
                        }
                    }
                }
            }

            None
        })
    }

    fn rebalance(self) -> Self {
        if !self.is_balanced() {
            let mut leaves: Vec<Self> = self.into_leaves().collect();
            return Self::merge(&mut leaves);
        }

        self
    }

    fn merge(leaves: &mut [Self]) -> Self {
        let range = leaves.len();

        match range {
            1 => std::mem::take(&mut leaves[0]),
            _ => {
                let mid = range / 2;
                let (left, right) = leaves.split_at_mut(mid);

                Self::node(Some(Self::merge(left)), Some(Self::merge(right)))
            }
        }
    }

    fn concat(self, other: Self) -> Self {
        Self::node(Some(self), Some(other)).rebalance()
    }

    fn join(self, other: Self) -> Self {
        if self.is_empty() {
            other
        } else if other.is_empty() {
            self
        } else {
            self.concat(other)
        }
    }

    fn char_at(&self, index: usize) -> Option<char> {
        match self {
            Self::Leaf(s) => s.get(index..)?.chars().next(),
            Self::Node {
                left,
                right,
                weight,
                ..
            } => {
                if index < *weight {
                    left.as_deref()
                        .expect("index < weight implies that left is Some")
                        .char_at(index)
                } else {
                    right.as_deref()?.char_at(index - weight)
                }
            }
        }
    }

    fn byte_at(&self, index: usize) -> Option<u8> {
        match self {
            Self::Leaf(s) => s.as_bytes().get(index).copied(),
            Self::Node {
                left,
                right,
                weight,
                ..
            } => {
                if index < *weight {
                    left.as_deref()
                        .expect("index < weight implies that left is Some")
                        .byte_at(index)
                } else {
                    right.as_deref()?.byte_at(index - weight)
                }
            }
        }
    }

    fn is_char_boundary(&self, index: usize) -> bool {
        if index == 0 || index == self.len() {
            return true;
        }

        if index > self.len() {
            return false;
        }

        matches!(self.byte_at(index), Some(b) if (b & 0b1100_0000) != 0b1000_0000)
    }

    pub fn next_char_boundary(&self, index: usize) -> Option<usize> {
        self.char_at(index).map(|c| index + c.len_utf8())
    }

    pub fn prev_char_boundary(&self, index: usize) -> Option<usize> {
        if index == 0 {
            return None;
        }

        let mut i = index.min(self.len()).saturating_sub(1);
        while !self.is_char_boundary(i) {
            i -= 1;
        }

        Some(i)
    }

    fn split(self, index: usize) -> (Option<Self>, Option<Self>) {
        match self {
            Self::Leaf(s) => {
                let (left_str, right_str) = s.split_at(index);
                let left_res = (!left_str.is_empty()).then(|| Self::leaf(left_str));
                let right_res = (!right_str.is_empty()).then(|| Self::leaf(right_str));

                (left_res, right_res)
            }
            Self::Node {
                left,
                right,
                weight,
                ..
            } => {
                let left_child = left.map(Rc::unwrap_or_clone);
                let right_child = right.map(Rc::unwrap_or_clone);

                match index.cmp(&weight) {
                    Ordering::Less => {
                        let l = left_child.expect("index < weight implies that left_child is Some");
                        let (split_left, split_right) = l.split(index);
                        (
                            split_left.map(Self::rebalance),
                            Some(Self::node(split_right, right_child).rebalance()),
                        )
                    }
                    Ordering::Greater => match right_child {
                        Some(r) => {
                            let (split_left, split_right) = r.split(index - weight);
                            (
                                Some(Self::node(left_child, split_left).rebalance()),
                                split_right.map(Self::rebalance),
                            )
                        }
                        None => (left_child, None),
                    },
                    Ordering::Equal => (left_child, right_child),
                }
            }
        }
    }

    pub fn insert_rope(self, index: usize, other: Rope) -> Self {
        if other.is_empty() {
            return self;
        }

        let start = index.min(self.len());
        let (left_tree, right_tree) = self.split(start);

        match (left_tree, right_tree) {
            (Some(l), Some(r)) => l.concat(other).concat(r),
            (Some(l), None) => l.concat(other),
            (None, Some(r)) => other.concat(r),
            (None, None) => other,
        }
    }

    pub fn insert_str(self, index: usize, str: &str) -> Self {
        self.insert_rope(index, Self::leaf(str))
    }

    pub fn remove(self, index: usize, len: usize) -> Self {
        let start = index.min(self.len());
        let end = start.saturating_add(len).min(self.len());

        if start >= end {
            return self;
        }

        let (lhs_l, lhs_r) = self.split(start);
        let rhs_r = lhs_r.and_then(|n| n.split(end - start).1);

        match (lhs_l, rhs_r) {
            (Some(l), Some(r)) => l.concat(r),
            (Some(l), None) => l,
            (None, Some(r)) => r,
            (None, None) => Self::default(),
        }
    }

    pub fn slice(&self, index: usize, len: usize) -> Self {
        let start = index.min(self.len());
        let end = start.saturating_add(len).min(self.len());

        if start >= end {
            return Self::default();
        }

        if start == 0 && end == self.len() {
            return self.clone();
        }

        match self {
            Self::Leaf(s) => {
                assert!(
                    s.is_char_boundary(start) && s.is_char_boundary(end),
                    "Tried slicing a string at invalid char boundary"
                );

                Self::leaf(&s[start..end])
            }
            Self::Node { left, right, .. } => {
                let l = left.as_deref();
                let r = right.as_deref();
                let l_len = l.map_or(0, Self::len);

                if end <= l_len {
                    l.map_or_default(|n| n.slice(start, end - start))
                } else if start >= l_len {
                    r.map_or_default(|n| n.slice(start - l_len, end - start))
                } else {
                    let a = l.map(|n| n.slice(start, l_len - start));
                    let b = r.map(|n| n.slice(0, end - l_len));
                    Self::node(a, b)
                }
            }
        }
    }

    pub fn slice_to_string(&self, index: usize, len: usize) -> String {
        self.slice(index, len).leaves_str().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(r: &Rope) -> String {
        r.slice_to_string(0, r.len())
    }

    fn opt_text(r: Option<Rope>) -> Option<String> {
        r.map(|r| text(&r))
    }

    mod fibonacci_sequence {
        use super::*;

        #[test]
        fn first_ten_values_are_correct() {
            let seq = fibonacci_sequence::<10>();
            assert_eq!(seq, [0, 1, 1, 2, 3, 5, 8, 13, 21, 34]);
        }
    }

    mod construction {
        use super::*;

        #[test]
        fn empty_default() {
            let r = Rope::default();
            assert!(r.is_leaf());
            assert!(!r.is_node());
            assert_eq!(r.len(), 0);
            assert_eq!(r.weight(), 0);
            assert_eq!(r.depth(), 0);
            assert_eq!(text(&r), "");
        }

        #[test]
        fn leaf_basic() {
            let r = Rope::leaf("hello");
            assert!(r.is_leaf());
            assert!(!r.is_node());
            assert_eq!(r.len(), 5);
            assert_eq!(r.weight(), 5);
            assert_eq!(r.depth(), 0);
            assert_eq!(text(&r), "hello");
        }

        #[test]
        fn leaf_getters_return_none() {
            let r = Rope::leaf("x");
            assert!(r.is_leaf());
            assert!(!r.is_node());
            assert!(r.get_left().is_none());
            assert!(r.get_right().is_none());
        }

        #[test]
        fn node_with_only_left_child() {
            let r = Rope::node(Some(Rope::leaf("abc")), None);
            assert!(!r.is_leaf());
            assert!(r.is_node());
            assert_eq!(r.len(), 3);
            assert_eq!(r.weight(), 3);
            assert_eq!(r.depth(), 1);
            assert_eq!(r.get_left().map(Rope::len), Some(3));
            assert!(r.get_right().is_none());
            assert_eq!(text(&r), "abc");
        }

        #[test]
        fn node_with_only_right_child() {
            let r = Rope::node(None, Some(Rope::leaf("abc")));
            assert!(!r.is_leaf());
            assert!(r.is_node());
            assert_eq!(r.len(), 3);
            assert_eq!(r.weight(), 0);
            assert_eq!(r.depth(), 1);
            assert!(r.get_left().is_none());
            assert_eq!(r.get_right().map(Rope::len), Some(3));
            assert_eq!(text(&r), "abc");
        }

        #[test]
        fn node_filters_empty_child() {
            let r = Rope::node(Some(Rope::leaf("")), Some(Rope::leaf("abc")));
            match &r {
                Rope::Node {
                    left,
                    right,
                    weight,
                    ..
                } => {
                    assert!(left.is_none());
                    assert!(right.is_some());
                    assert_eq!(*weight, 0);
                }
                _ => unreachable!("`r` is a node"),
            }
            assert_eq!(text(&r), "abc");
        }

        #[test]
        #[should_panic]
        fn node_with_both_children_empty_panics() {
            let _ = Rope::node(Some(Rope::leaf("")), Some(Rope::leaf("")));
        }

        #[test]
        fn is_empty_on_default() {
            assert!(Rope::default().is_empty());
        }

        #[test]
        fn is_empty_on_leaf() {
            assert!(!Rope::leaf("x").is_empty());
        }

        #[test]
        fn is_empty_after_remove_all() {
            let r = Rope::leaf("hello").remove(0, 5);
            assert!(r.is_empty());
        }
    }

    mod concatenation {
        use super::*;

        #[test]
        fn concat_two_leaves() {
            let r = Rope::leaf("foo").concat(Rope::leaf("bar"));
            assert_eq!(r.len(), 6);
            assert_eq!(r.weight(), 3);
            assert_eq!(r.depth(), 1);
            assert!(r.is_balanced());
            assert_eq!(text(&r), "foobar");
        }

        #[test]
        fn concat_empty_left() {
            let r = Rope::default().concat(Rope::leaf("foo"));
            assert_eq!(r.len(), 3);
            assert_eq!(text(&r), "foo");
        }

        #[test]
        fn concat_empty_right() {
            let r = Rope::leaf("foo").concat(Rope::default());
            assert_eq!(r.len(), 3);
            assert_eq!(text(&r), "foo");
        }

        #[test]
        #[should_panic]
        fn concat_two_empty_panics() {
            let _ = Rope::default().concat(Rope::default());
        }

        #[test]
        fn join_two_nonempty() {
            let r = Rope::leaf("foo").join(Rope::leaf("bar"));
            assert_eq!(text(&r), "foobar");
            assert_eq!(r.len(), 6);
        }

        #[test]
        fn join_empty_left() {
            let r = Rope::default().join(Rope::leaf("foo"));
            assert_eq!(text(&r), "foo");
        }

        #[test]
        fn join_empty_right() {
            let r = Rope::leaf("foo").join(Rope::default());
            assert_eq!(text(&r), "foo");
        }

        #[test]
        fn join_two_empty_is_noop() {
            let r = Rope::default().join(Rope::default());
            assert!(r.is_empty());
        }
    }

    mod equality {
        use super::*;

        #[test]
        fn eq_two_defaults() {
            assert_eq!(Rope::default(), Rope::default());
        }

        #[test]
        fn eq_two_identical_leaves() {
            assert_eq!(Rope::leaf("hello"), Rope::leaf("hello"));
        }

        #[test]
        fn ne_different_leaves() {
            assert_ne!(Rope::leaf("hello"), Rope::leaf("world"));
        }

        #[test]
        fn eq_tree_and_leaf_with_same_text_are_not_equal() {
            let tree = Rope::leaf("foo").concat(Rope::leaf("bar"));
            let leaf = Rope::leaf("foobar");
            assert_ne!(tree, leaf);
        }
    }

    mod char_at {
        use super::*;

        #[test]
        fn ascii() {
            let r = Rope::leaf("hello");
            assert_eq!(r.char_at(0), Some('h'));
            assert_eq!(r.char_at(1), Some('e'));
            assert_eq!(r.char_at(4), Some('o'));
            assert_eq!(r.char_at(5), None);
        }

        #[test]
        fn unicode() {
            let r = Rope::leaf("héllo");
            assert_eq!(r.len(), 6);
            assert_eq!(r.char_at(0), Some('h'));
            assert_eq!(r.char_at(1), Some('é'));
            assert_eq!(r.char_at(2), None);
            assert_eq!(r.char_at(3), Some('l'));
            assert_eq!(r.char_at(5), Some('o'));
            assert_eq!(r.char_at(6), None);
        }

        #[test]
        fn across_leaves() {
            let r = Rope::leaf("foo").concat(Rope::leaf("bar"));
            assert_eq!(r.char_at(0), Some('f'));
            assert_eq!(r.char_at(2), Some('o'));
            assert_eq!(r.char_at(3), Some('b'));
            assert_eq!(r.char_at(5), Some('r'));
            assert_eq!(r.char_at(6), None);
        }

        #[test]
        fn multibyte_across_leaves() {
            let r = Rope::leaf("ab").concat(Rope::leaf("é"));
            assert_eq!(r.len(), 4);
            assert_eq!(r.char_at(0), Some('a'));
            assert_eq!(r.char_at(1), Some('b'));
            assert_eq!(r.char_at(2), Some('é'));
            assert_eq!(r.char_at(3), None);
        }

        #[test]
        fn left_only_node_past_end_is_none() {
            let r = Rope::node(Some(Rope::leaf("abc")), None);
            assert_eq!(r.char_at(0), Some('a'));
            assert_eq!(r.char_at(2), Some('c'));
            assert_eq!(r.char_at(3), None);
            assert_eq!(r.char_at(100), None);
        }

        #[test]
        fn right_only_node() {
            let r = Rope::node(None, Some(Rope::leaf("abc")));
            assert_eq!(r.char_at(0), Some('a'));
            assert_eq!(r.char_at(2), Some('c'));
            assert_eq!(r.char_at(3), None);
        }

        #[test]
        #[should_panic]
        fn slice_starts_at_invalid_char_boundary_panics() {
            let r = Rope::leaf("héllo");
            let _ = r.slice(2, 1);
        }

        #[test]
        #[should_panic]
        fn slice_ends_at_invalid_char_boundary_panics() {
            let r = Rope::leaf("héllo");
            let _ = r.slice(1, 1);
        }
    }

    mod byte_at {
        use super::*;

        #[test]
        fn ascii_leaf() {
            let r = Rope::leaf("abc");
            assert_eq!(r.byte_at(0), Some(b'a'));
            assert_eq!(r.byte_at(1), Some(b'b'));
            assert_eq!(r.byte_at(2), Some(b'c'));
            assert_eq!(r.byte_at(3), None);
        }

        #[test]
        fn out_of_bounds_is_none() {
            let r = Rope::leaf("abc");
            assert_eq!(r.byte_at(100), None);
        }

        #[test]
        fn empty_leaf_yields_none() {
            let r = Rope::default();
            assert_eq!(r.byte_at(0), None);
        }

        #[test]
        fn multibyte_bytes_are_raw_utf8() {
            let r = Rope::leaf("é");
            assert_eq!(r.byte_at(0), Some(0xC3));
            assert_eq!(r.byte_at(1), Some(0xA9));
            assert_eq!(r.byte_at(2), None);
        }

        #[test]
        fn across_leaves() {
            let r = Rope::leaf("ab").concat(Rope::leaf("cd"));
            assert_eq!(r.byte_at(0), Some(b'a'));
            assert_eq!(r.byte_at(1), Some(b'b'));
            assert_eq!(r.byte_at(2), Some(b'c'));
            assert_eq!(r.byte_at(3), Some(b'd'));
            assert_eq!(r.byte_at(4), None);
        }

        #[test]
        fn on_node_boundary_index() {
            let r = Rope::leaf("ab").concat(Rope::leaf("cd"));
            assert_eq!(r.byte_at(2), Some(b'c'));
        }

        #[test]
        fn right_only_node() {
            let r = Rope::node(None, Some(Rope::leaf("xy")));
            assert_eq!(r.byte_at(0), Some(b'x'));
            assert_eq!(r.byte_at(1), Some(b'y'));
            assert_eq!(r.byte_at(2), None);
        }

        #[test]
        fn left_only_node() {
            let r = Rope::node(Some(Rope::leaf("xy")), None);
            assert_eq!(r.byte_at(0), Some(b'x'));
            assert_eq!(r.byte_at(1), Some(b'y'));
            assert_eq!(r.byte_at(2), None);
        }
    }

    mod is_char_boundary {
        use super::*;

        #[test]
        fn zero_is_always_boundary() {
            assert!(Rope::leaf("hello").is_char_boundary(0));
            assert!(Rope::default().is_char_boundary(0));
        }

        #[test]
        fn len_is_always_boundary() {
            let r = Rope::leaf("hello");
            assert!(r.is_char_boundary(5));
        }

        #[test]
        fn past_len_is_not_boundary() {
            let r = Rope::leaf("hello");
            assert!(!r.is_char_boundary(6));
            assert!(!r.is_char_boundary(100));
        }

        #[test]
        fn ascii_positions_are_all_boundaries() {
            let r = Rope::leaf("hello");
            for i in 0..=5 {
                assert!(r.is_char_boundary(i), "index {} should be a boundary", i);
            }
        }

        #[test]
        fn multibyte_middle_is_not_boundary() {
            let r = Rope::leaf("héllo");
            assert!(r.is_char_boundary(0));
            assert!(r.is_char_boundary(1));
            assert!(!r.is_char_boundary(2));
            assert!(r.is_char_boundary(3));
            assert!(r.is_char_boundary(4));
            assert!(r.is_char_boundary(5));
            assert!(r.is_char_boundary(6));
        }

        #[test]
        fn across_leaves() {
            let r = Rope::leaf("ab").concat(Rope::leaf("é"));
            assert!(r.is_char_boundary(0));
            assert!(r.is_char_boundary(1));
            assert!(r.is_char_boundary(2));
            assert!(!r.is_char_boundary(3));
            assert!(r.is_char_boundary(4));
        }

        #[test]
        fn empty_rope_past_zero_is_not_boundary() {
            let r = Rope::default();
            assert!(!r.is_char_boundary(1));
        }
    }

    mod next_char_boundary {
        use super::*;

        #[test]
        fn ascii_moves_one_byte() {
            let r = Rope::leaf("hello");
            assert_eq!(r.next_char_boundary(0), Some(1));
            assert_eq!(r.next_char_boundary(1), Some(2));
            assert_eq!(r.next_char_boundary(4), Some(5));
        }

        #[test]
        fn at_end_is_none() {
            let r = Rope::leaf("hello");
            assert_eq!(r.next_char_boundary(5), None);
        }

        #[test]
        fn past_end_is_none() {
            let r = Rope::leaf("hello");
            assert_eq!(r.next_char_boundary(100), None);
        }

        #[test]
        fn multibyte_skips_whole_char() {
            let r = Rope::leaf("héllo");
            assert_eq!(r.next_char_boundary(0), Some(1));
            assert_eq!(r.next_char_boundary(1), Some(3));
            assert_eq!(r.next_char_boundary(3), Some(4));
        }

        #[test]
        fn mid_char_is_none() {
            let r = Rope::leaf("héllo");
            assert_eq!(r.next_char_boundary(2), None);
        }

        #[test]
        fn across_leaves() {
            let r = Rope::leaf("ab").concat(Rope::leaf("cd"));
            assert_eq!(r.next_char_boundary(0), Some(1));
            assert_eq!(r.next_char_boundary(1), Some(2));
            assert_eq!(r.next_char_boundary(2), Some(3));
            assert_eq!(r.next_char_boundary(3), Some(4));
            assert_eq!(r.next_char_boundary(4), None);
        }

        #[test]
        fn empty_rope_zero_is_none() {
            let r = Rope::default();
            assert_eq!(r.next_char_boundary(0), None);
        }

        #[test]
        fn four_byte_char() {
            let r = Rope::leaf("😀");
            assert_eq!(r.len(), 4);
            assert_eq!(r.next_char_boundary(0), Some(4));
        }
    }

    mod prev_char_boundary {
        use super::*;

        #[test]
        fn ascii_moves_back_one() {
            let r = Rope::leaf("hello");
            assert_eq!(r.prev_char_boundary(1), Some(0));
            assert_eq!(r.prev_char_boundary(5), Some(4));
        }

        #[test]
        fn at_zero_is_none() {
            let r = Rope::leaf("hello");
            assert_eq!(r.prev_char_boundary(0), None);
        }

        #[test]
        fn multibyte_lands_on_char_start() {
            let r = Rope::leaf("héllo");
            assert_eq!(r.prev_char_boundary(3), Some(1));
            assert_eq!(r.prev_char_boundary(1), Some(0));
            assert_eq!(r.prev_char_boundary(4), Some(3));
        }

        #[test]
        fn across_leaves() {
            let r = Rope::leaf("ab").concat(Rope::leaf("cd"));
            assert_eq!(r.prev_char_boundary(4), Some(3));
            assert_eq!(r.prev_char_boundary(2), Some(1));
            assert_eq!(r.prev_char_boundary(1), Some(0));
        }

        #[test]
        fn crossing_leaf_boundary_with_multibyte() {
            let r = Rope::leaf("ab").concat(Rope::leaf("é"));
            assert_eq!(r.prev_char_boundary(4), Some(2));
            assert_eq!(r.prev_char_boundary(2), Some(1));
        }

        #[test]
        fn empty_rope_zero_is_none() {
            let r = Rope::default();
            assert_eq!(r.prev_char_boundary(0), None);
        }

        #[test]
        fn four_byte_char() {
            let r = Rope::leaf("a😀");
            assert_eq!(r.prev_char_boundary(5), Some(1));
            assert_eq!(r.prev_char_boundary(1), Some(0));
        }

        #[test]
        fn multibyte_at_start() {
            let r = Rope::leaf("éllo");
            assert_eq!(r.prev_char_boundary(2), Some(0));
            assert_eq!(r.prev_char_boundary(1), Some(0));
            assert_eq!(r.prev_char_boundary(0), None);
        }

        #[test]
        fn out_of_bounds_clamps_to_last_boundary() {
            let r = Rope::leaf("hello");
            assert_eq!(r.prev_char_boundary(100), Some(4));
        }

        #[test]
        fn out_of_bounds_with_multibyte() {
            let r = Rope::leaf("héllo");
            assert_eq!(r.prev_char_boundary(100), Some(5));
            assert_eq!(r.prev_char_boundary(6), Some(5));
        }

        #[test]
        fn next_then_prev_is_identity() {
            let r = Rope::leaf("héllo wörld");
            let mut i = 0;
            while i < r.len() {
                if r.is_char_boundary(i) {
                    let next = r
                        .next_char_boundary(i)
                        .expect("i < r.len() implies that next boundary exists");

                    assert_eq!(r.prev_char_boundary(next), Some(i));
                }
                i += 1;
            }
        }
    }

    mod split {
        use super::*;

        #[test]
        fn leaf_middle() {
            let r = Rope::leaf("hello");
            let (l, r) = r.split(2);
            assert_eq!(opt_text(l), Some("he".to_string()));
            assert_eq!(opt_text(r), Some("llo".to_string()));
        }

        #[test]
        fn at_zero() {
            let r = Rope::leaf("hello");
            let (l, r) = r.split(0);
            assert_eq!(l, None);
            assert_eq!(opt_text(r), Some("hello".to_string()));
        }

        #[test]
        fn at_end() {
            let r = Rope::leaf("hello");
            let (l, r) = r.split(5);
            assert_eq!(opt_text(l), Some("hello".to_string()));
            assert_eq!(r, None);
        }

        #[test]
        fn node() {
            let r = Rope::leaf("foo").concat(Rope::leaf("bar"));
            let (l, r) = r.split(4);
            assert_eq!(opt_text(l), Some("foob".to_string()));
            assert_eq!(opt_text(r), Some("ar".to_string()));
        }

        #[test]
        fn empty() {
            let r = Rope::default();
            let (l, r) = r.split(0);
            assert_eq!(l, None);
            assert_eq!(r, None);
        }

        #[test]
        fn beyond_end_on_left_only_node() {
            let r = Rope::node(Some(Rope::leaf("abc")), None);
            let (l, r) = r.split(10);
            assert_eq!(opt_text(l), Some("abc".to_string()));
            assert_eq!(r, None);
        }

        #[test]
        fn on_right_only_node_triggers_single_leaf_rebalance() {
            let r = Rope::node(None, Some(Rope::leaf("abc")));
            let (l, r) = r.split(1);
            assert_eq!(l, Some(Rope::leaf("a")));
            assert_eq!(r, Some(Rope::leaf("bc")));
            assert_eq!(opt_text(l), Some("a".to_string()));
            assert_eq!(opt_text(r), Some("bc".to_string()));
        }

        #[test]
        fn node_at_zero_hits_none_left() {
            let r = Rope::leaf("foo").concat(Rope::leaf("bar"));
            let (l, r) = r.split(0);
            assert_eq!(l, None);
            assert_eq!(opt_text(r), Some("foobar".to_string()));
        }

        #[test]
        fn node_exactly_at_weight() {
            let r = Rope::leaf("foo").concat(Rope::leaf("bar"));
            let (l, r) = r.split(3);
            assert_eq!(opt_text(l), Some("foo".to_string()));
            assert_eq!(opt_text(r), Some("bar".to_string()));
        }

        #[test]
        fn node_at_end_hits_none_right() {
            let r = Rope::leaf("foo").concat(Rope::leaf("bar"));
            let (l, r) = r.split(6);
            assert_eq!(opt_text(l), Some("foobar".to_string()));
            assert_eq!(r, None);
        }
    }

    mod insert_str {
        use super::*;

        #[test]
        fn into_empty() {
            let r = Rope::default().insert_str(0, "hello");
            assert_eq!(text(&r), "hello");
        }

        #[test]
        fn at_beginning() {
            let r = Rope::leaf("world").insert_str(0, "hello ");
            assert_eq!(text(&r), "hello world");
        }

        #[test]
        fn at_middle() {
            let r = Rope::leaf("helloworld").insert_str(5, " ");
            assert_eq!(text(&r), "hello world");
        }

        #[test]
        fn at_end() {
            let r = Rope::leaf("hello").insert_str(5, " world");
            assert_eq!(text(&r), "hello world");
        }

        #[test]
        fn out_of_bounds_clamps() {
            let r = Rope::leaf("hello").insert_str(100, "!");
            assert_eq!(text(&r), "hello!");
        }

        #[test]
        fn empty_text_is_noop() {
            let r = Rope::leaf("hello").insert_str(2, "");
            assert_eq!(text(&r), "hello");
        }

        #[test]
        fn at_front_hits_none_left_arm() {
            let r = Rope::leaf("abc").insert_str(0, "x");
            assert_eq!(text(&r), "xabc");
        }

        #[test]
        fn at_back_hits_none_right_arm() {
            let r = Rope::leaf("abc").insert_str(3, "x");
            assert_eq!(text(&r), "abcx");
        }

        #[test]
        fn before_multibyte_char() {
            let r = Rope::leaf("héllo").insert_str(1, "X");
            assert_eq!(text(&r), "hXéllo");
        }

        #[test]
        fn after_multibyte_char() {
            let r = Rope::leaf("héllo").insert_str(3, "X");
            assert_eq!(text(&r), "héXllo");
        }

        #[test]
        fn multibyte_str() {
            let r = Rope::leaf("αβγ").insert_str(2, "X");
            assert_eq!(text(&r), "αXβγ");
        }
    }

    mod insert_rope {
        use super::*;

        #[test]
        fn empty_other_returns_self_unchanged() {
            let r = Rope::leaf("hello");
            let r2 = r.clone().insert_rope(3, Rope::default());
            assert_eq!(r, r2);
        }

        #[test]
        fn empty_other_on_empty_self() {
            let r = Rope::default().insert_rope(0, Rope::default());
            assert!(r.is_empty());
        }

        #[test]
        fn into_empty_self() {
            let r = Rope::leaf("hello");
            let r2 = Rope::default().insert_rope(0, r.clone());
            assert_eq!(r, r2);
            assert_eq!(text(&r2), "hello");
        }

        #[test]
        fn at_beginning() {
            let r = Rope::leaf("world").insert_rope(0, Rope::leaf("hello "));
            assert_eq!(text(&r), "hello world");
        }

        #[test]
        fn at_middle() {
            let r = Rope::leaf("helloworld").insert_rope(5, Rope::leaf(" "));
            assert_eq!(text(&r), "hello world");
        }

        #[test]
        fn at_end() {
            let r = Rope::leaf("hello").insert_rope(5, Rope::leaf(" world"));
            assert_eq!(text(&r), "hello world");
        }

        #[test]
        fn out_of_bounds_clamps() {
            let r = Rope::leaf("hello").insert_rope(100, Rope::leaf("!"));
            assert_eq!(text(&r), "hello!");
        }

        #[test]
        fn multi_leaf_other() {
            let other = Rope::leaf("foo").concat(Rope::leaf("bar"));
            let r = Rope::leaf("X").insert_rope(1, other);
            assert_eq!(text(&r), "Xfoobar");
        }

        #[test]
        fn insert_into_multi_leaf_self() {
            let base = Rope::leaf("aaa").concat(Rope::leaf("bbb"));
            let r = base.insert_rope(3, Rope::leaf("---"));
            assert_eq!(text(&r), "aaa---bbb");
        }

        #[test]
        fn clipboard_round_trip() {
            let doc = Rope::leaf("hello world");
            let clip = doc.slice(6, 5);
            let pasted = doc.clone().insert_rope(0, clip);

            assert_eq!(text(&pasted), "worldhello world");
            assert_eq!(text(&doc), "hello world");
        }

        #[test]
        fn multibyte_rope() {
            let r = Rope::leaf("héllo").insert_rope(1, Rope::leaf("X"));
            assert_eq!(text(&r), "hXéllo");
        }

        #[test]
        fn self_and_other_from_same_tree() {
            let r = Rope::leaf("foobar");
            let left = r.slice(0, 3);
            let right = r.slice(3, 3);
            let combined = left.insert_rope(3, right);
            assert_eq!(text(&combined), "foobar");
        }

        #[test]
        fn other_is_descendant_of_self() {
            let doc = Rope::leaf("abcdef");
            let clip = doc.slice(2, 2);
            let pasted = doc.insert_rope(4, clip);
            assert_eq!(text(&pasted), "abcdcdef");
        }
    }

    mod remove {
        use super::*;

        #[test]
        fn middle() {
            let r = Rope::leaf("hello world").remove(5, 1);
            assert_eq!(text(&r), "helloworld");
        }

        #[test]
        fn whole() {
            let r = Rope::leaf("hello").remove(0, 5);
            assert_eq!(r.len(), 0);
            assert_eq!(text(&r), "");
        }

        #[test]
        fn beyond_end_clamps() {
            let r = Rope::leaf("hello").remove(3, 100);
            assert_eq!(text(&r), "hel");
        }

        #[test]
        fn zero_len() {
            let r = Rope::leaf("hello").remove(2, 0);
            assert_eq!(text(&r), "hello");
        }

        #[test]
        fn entirely_out_of_bounds() {
            let r = Rope::leaf("hello").remove(100, 5);
            assert_eq!(text(&r), "hello");
        }

        #[test]
        fn from_beginning_hits_none_left_arm() {
            let r = Rope::leaf("abc").remove(0, 1);
            assert_eq!(text(&r), "bc");
        }

        #[test]
        fn from_end_hits_none_right_arm() {
            let r = Rope::leaf("abc").remove(2, 1);
            assert_eq!(text(&r), "ab");
        }

        #[test]
        fn spanning_three_leaves() {
            let r = Rope::leaf("aaa")
                .concat(Rope::leaf("bbb"))
                .concat(Rope::leaf("ccc"));
            let out = r.remove(2, 5);
            assert_eq!(text(&out), "aacc");
        }

        #[test]
        fn multibyte_char() {
            let r = Rope::leaf("héllo").remove(1, 2);
            assert_eq!(text(&r), "hllo");
        }
    }

    mod slice {
        use super::*;

        #[test]
        fn full() {
            let r = Rope::leaf("hello");
            let s = r.slice(0, 5);
            assert_eq!(text(&s), "hello");
        }

        #[test]
        fn middle() {
            let r = Rope::leaf("hello world");
            let s = r.slice(6, 5);
            assert_eq!(text(&s), "world");
        }

        #[test]
        fn out_of_bounds_clamps() {
            let r = Rope::leaf("hello");
            let s = r.slice(3, 100);
            assert_eq!(text(&s), "lo");
        }

        #[test]
        fn empty_range() {
            let r = Rope::leaf("hello");
            let s = r.slice(2, 0);
            assert_eq!(text(&s), "");
            assert_eq!(s.len(), 0);
        }

        #[test]
        fn across_leaves() {
            let r = Rope::leaf("foo")
                .concat(Rope::leaf("bar"))
                .concat(Rope::leaf("baz"));
            let s = r.slice(2, 5);
            assert_eq!(text(&s), "obarb");
        }

        #[test]
        fn full_range_keeps_content() {
            let r = Rope::leaf("hello").concat(Rope::leaf("world"));
            let s = r.slice(0, 10);
            assert_eq!(text(&s), "helloworld");
        }

        #[test]
        fn hits_left_branch() {
            let r = Rope::leaf("aaa").concat(Rope::leaf("bbb"));
            assert_eq!(text(&r.slice(0, 2)), "aa");
        }

        #[test]
        fn hits_right_branch() {
            let r = Rope::leaf("aaa").concat(Rope::leaf("bbb"));
            assert_eq!(text(&r.slice(4, 2)), "bb");
        }

        #[test]
        fn hits_straddle_branch() {
            let r = Rope::leaf("aaa").concat(Rope::leaf("bbb"));
            assert_eq!(text(&r.slice(2, 2)), "ab");
        }

        #[test]
        fn hits_left_branch_with_nonzero_start() {
            let r = Rope::leaf("aaa").concat(Rope::leaf("bbb"));
            assert_eq!(text(&r.slice(1, 1)), "a");
        }

        #[test]
        fn hits_right_branch_with_interior_end() {
            let r = Rope::leaf("aaa").concat(Rope::leaf("bbb"));
            assert_eq!(text(&r.slice(4, 1)), "b");
        }

        #[test]
        fn on_right_only_node() {
            let r = Rope::node(None, Some(Rope::leaf("abc")));
            assert_eq!(text(&r.slice(1, 1)), "b");
        }

        #[test]
        fn multibyte_across_leaves() {
            let r = Rope::leaf("αβγ").concat(Rope::leaf("δεζ"));
            assert_eq!(r.slice_to_string(0, 8), "αβγδ");
            assert_eq!(r.slice_to_string(4, 8), "γδεζ");
        }

        #[test]
        fn does_not_mutate_source() {
            let r = Rope::leaf("hello world");
            let _ = r.slice(0, 5);
            let _ = r.slice(6, 5);
            assert_eq!(text(&r), "hello world");
        }
    }

    mod slice_to_string {
        use super::*;

        #[test]
        fn basic() {
            let r = Rope::leaf("hello world");
            assert_eq!(r.slice_to_string(6, 5), "world");
        }

        #[test]
        fn unicode() {
            let r = Rope::leaf("héllo wörld");
            assert_eq!(r.slice_to_string(0, 7), "héllo ");
            assert_eq!(r.slice_to_string(7, 6), "wörld");
        }

        #[test]
        fn out_of_bounds_clamps() {
            let r = Rope::leaf("hi");
            assert_eq!(r.slice_to_string(1, 100), "i");
        }

        #[test]
        fn does_not_mutate_source() {
            let r = Rope::leaf("hello world");
            let _ = r.slice_to_string(0, 5);
            let _ = r.slice_to_string(6, 5);
            assert_eq!(text(&r), "hello world");
        }
    }

    mod display {
        use super::*;

        #[test]
        fn empty_rope_displays_empty() {
            assert_eq!(format!("{}", Rope::default()), "");
        }

        #[test]
        fn leaf_displays_content() {
            assert_eq!(format!("{}", Rope::leaf("hello")), "hello");
        }

        #[test]
        fn node_displays_concatenated_content() {
            let r = Rope::leaf("foo").concat(Rope::leaf("bar"));
            assert_eq!(format!("{}", r), "foobar");
        }

        #[test]
        fn deeper_tree_displays_in_order() {
            let r = Rope::leaf("a")
                .concat(Rope::leaf("b"))
                .concat(Rope::leaf("c"))
                .concat(Rope::leaf("d"));
            assert_eq!(format!("{}", r), "abcd");
        }

        #[test]
        fn multibyte_displays_correctly() {
            let r = Rope::leaf("héllo").concat(Rope::leaf(" wörld"));
            assert_eq!(format!("{}", r), "héllo wörld");
        }

        #[test]
        fn display_matches_slice_to_string() {
            let r = Rope::leaf("foo")
                .concat(Rope::leaf("bar"))
                .concat(Rope::leaf("baz"));
            assert_eq!(format!("{}", r), r.slice_to_string(0, r.len()));
        }

        #[test]
        fn display_with_control_chars_passes_through() {
            let r = Rope::leaf("line1\nline2\ttabbed");
            assert_eq!(format!("{}", r), "line1\nline2\ttabbed");
        }

        #[test]
        fn to_string_works_via_blanket_impl() {
            let r = Rope::leaf("hello").concat(Rope::leaf(" world"));
            assert_eq!(r.to_string(), "hello world");
        }

        #[test]
        fn display_single_child_node() {
            let r = Rope::node(Some(Rope::leaf("abc")), None);
            assert_eq!(format!("{}", r), "abc");
        }
    }

    mod structural_sharing {
        use super::*;

        #[test]
        fn clone_shares_leaf_allocation() {
            let a = Rope::leaf("hello");
            let b = a.clone();
            match (&a, &b) {
                (Rope::Leaf(ra), Rope::Leaf(rb)) => assert!(Rc::ptr_eq(ra, rb)),
                _ => unreachable!("`a` and `b` are leaves"),
            }
        }

        #[test]
        fn clone_shares_node_children() {
            let a = Rope::leaf("foo").concat(Rope::leaf("bar"));
            let b = a.clone();
            match (&a, &b) {
                (
                    Rope::Node {
                        left: la,
                        right: ra,
                        ..
                    },
                    Rope::Node {
                        left: lb,
                        right: rb,
                        ..
                    },
                ) => {
                    assert!(Rc::ptr_eq(la.as_ref().unwrap(), lb.as_ref().unwrap()));
                    assert!(Rc::ptr_eq(ra.as_ref().unwrap(), rb.as_ref().unwrap()));
                }
                _ => unreachable!("`a` and `b` are nodes"),
            }
        }

        #[test]
        fn slice_full_range_shares_root() {
            let a = Rope::leaf("foo").concat(Rope::leaf("bar"));
            let b = a.slice(0, 6);
            match (&a, &b) {
                (
                    Rope::Node {
                        left: la,
                        right: ra,
                        ..
                    },
                    Rope::Node {
                        left: lb,
                        right: rb,
                        ..
                    },
                ) => {
                    assert!(Rc::ptr_eq(la.as_ref().unwrap(), lb.as_ref().unwrap()));
                    assert!(Rc::ptr_eq(ra.as_ref().unwrap(), rb.as_ref().unwrap()));
                }
                _ => unreachable!("`a` and `b` are nodes"),
            }
        }
    }

    mod rebalance {
        use super::*;

        #[test]
        fn flattens_single_leaf_chain() {
            let r = Rope::node(Some(Rope::leaf("a")), None);
            assert!(!r.is_balanced());

            let r = r.rebalance();
            assert!(r.is_balanced());
            assert_eq!(r, Rope::leaf("a"));
        }

        #[test]
        fn merge_three_leaves_rebuilds_balanced_tree() {
            let ab = Rope::leaf("a").concat(Rope::leaf("b"));
            let r = ab.concat(Rope::leaf("c"));
            assert_eq!(text(&r), "abc");
            assert!(r.is_balanced());
        }

        #[test]
        fn merge_two_leaves_arm() {
            let r = Rope::node(Some(Rope::leaf("a")), None).concat(Rope::leaf("b"));
            assert_eq!(text(&r), "ab");
            assert!(r.is_balanced());
        }

        #[test]
        fn is_balanced_false_when_depth_exceeds_table() {
            let mut r = Rope::leaf("x");
            for _ in 0..92 {
                r = Rope::node(Some(r), None);
            }
            assert_eq!(r.depth(), 92);
            assert!(!r.is_balanced());
        }

        #[test]
        fn into_leaves_handles_single_child_node() {
            let r = Rope::node(Some(Rope::leaf("hello")), None);
            let leaves: Vec<_> = r.into_leaves().collect();
            assert_eq!(leaves.len(), 1);
        }

        #[test]
        fn leaves_iterates_right_only_node() {
            let r = Rope::node(None, Some(Rope::leaf("hello")));
            let leaves: Vec<_> = r.leaves().collect();
            assert_eq!(leaves.len(), 1);

            match leaves[0] {
                Rope::Leaf(s) => assert_eq!(&**s, "hello"),
                _ => unreachable!("`leaves` contains only leaves"),
            }
        }
    }

    mod stress {
        use super::*;

        #[test]
        fn many_appends_keep_text_correct() {
            let mut r = Rope::default();
            for i in 0..100 {
                let s = format!("{}", i % 10);
                let r_len = r.len();
                r = r.insert_str(r_len, &s);
            }
            assert_eq!(r.len(), 100);
            assert_eq!(
                r.slice_to_string(0, 100),
                (0..100)
                    .map(|i| char::from(b'0' + (i % 10) as u8))
                    .collect::<String>()
            );
        }

        #[test]
        fn round_trip_insert_then_slice() {
            let r = Rope::leaf("hello world").insert_str(5, ",");
            assert_eq!(r.slice_to_string(0, 12), "hello, world");
        }

        #[test]
        fn edit_sequence_preserves_content() {
            let mut r = Rope::leaf("the quick brown fox");
            r = r.insert_str(4, "very ");
            r = r.remove(0, 4);
            assert_eq!(text(&r), "very quick brown fox");
        }

        #[test]
        fn empty_ops_do_not_panic() {
            let r = Rope::default();
            assert_eq!(text(&r.clone().insert_str(0, "")), "");
            assert_eq!(text(&r.clone().remove(0, 0)), "");
            assert_eq!(text(&r.clone().slice(0, 0)), "");
        }

        #[test]
        fn random_edits_match_string_model() {
            let mut rope = Rope::default();
            let mut model = String::new();
            let mut seed: u64 = 0x9E37_79B9_7F4A_7C15;
            let mut rng = || {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                seed
            };

            for step in 0..1000 {
                match rng() % 3 {
                    0 => {
                        let pos = (rng() as usize) % (model.len() + 1);
                        let byte = b'a' + (rng() % 26) as u8;
                        let text = (byte as char).to_string();
                        rope = rope.insert_str(pos, &text);
                        model.insert_str(pos, &text);
                    }
                    1 => {
                        if !model.is_empty() {
                            let pos = (rng() as usize) % model.len();
                            let max = (model.len() - pos).min(8);
                            let len = ((rng() as usize) % max) + 1;
                            rope = rope.remove(pos, len);
                            model.replace_range(pos..pos + len, "");
                        }
                    }
                    _ => {
                        if model.len() >= 4 {
                            let pos = (rng() as usize) % (model.len() - 3);
                            let len = ((rng() as usize) % 3) + 1;
                            let chunk = model[pos..pos + len].to_string();

                            rope = rope.remove(pos, len);
                            model.replace_range(pos..pos + len, "");

                            let dst = (rng() as usize) % (model.len() + 1);
                            rope = rope.insert_str(dst, &chunk);
                            model.insert_str(dst, &chunk);
                        }
                    }
                }

                assert_eq!(rope.len(), model.len(), "len mismatch at step {}", step);
                assert_eq!(
                    rope.slice_to_string(0, rope.len()),
                    model,
                    "content mismatch at step {}",
                    step
                );
            }
        }

        #[test]
        fn deep_tree_stays_balanced_after_many_inserts() {
            let mut r = Rope::default();
            for i in 0..1000 {
                let s = format!("{}", (b'a' + (i % 26) as u8) as char);
                let len = r.len();
                r = r.insert_str(len, &s);
                assert!(r.is_balanced(), "unbalanced after {} inserts", i + 1);
            }
            assert_eq!(r.len(), 1000);
        }
    }
}
