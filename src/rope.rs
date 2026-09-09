use std::rc::Rc;

const fn fibonacci_sequence<const N: usize>() -> [usize; N] {
    let mut seq = [0; N];

    let mut i = 0;
    while i < N {
        if i < 2 {
            seq[i] = 1;
        } else {
            seq[i] = seq[i - 1] + seq[i - 2];
        }

        i += 1;
    }

    seq
}

const FIBONACCI_SEQUENCE: [usize; 93] = fibonacci_sequence::<93>();

#[derive(Debug, Clone)]
pub enum Rope {
    Leaf(Rc<str>),
    Node {
        left: Option<Rc<Self>>,
        right: Option<Rc<Self>>,
        weight: usize,
        len: usize,
    },
}

impl Default for Rope {
    fn default() -> Self {
        Self::Leaf(Rc::from(""))
    }
}

impl Rope {
    fn filter_empty(node: Option<Self>) -> Option<Self> {
        match node {
            Some(Self::Leaf(s)) if s.is_empty() => None,
            other => other
        }
    }

    pub fn leaf(s: String) -> Self {
        Self::Leaf(Rc::from(s))
    }

    pub fn node(left: Option<Self>, right: Option<Self>) -> Self {
        let left_res = Self::filter_empty(left);
        let right_res = Self::filter_empty(right);

        assert!(
            left_res.is_some() || right_res.is_some(),
            "Can't have an internal node with empty children. Use Rope::default or Rope::leaf"
        );

        let left_len = left_res.as_ref().map_or(0, Self::len);
        let right_len = right_res.as_ref().map_or(0, Self::len);

        Self::Node {
            weight: left_len,
            len: left_len + right_len,
            left: left_res.map(Rc::from),
            right: right_res.map(Rc::from),
        }
    }

    pub fn get_left(&self) -> Option<&Self> {
        match self {
            Self::Leaf(_) => None,
            Self::Node { left, .. } => left.as_deref(),
        }
    }

    pub fn get_right(&self) -> Option<&Self> {
        match self {
            Self::Leaf(_) => None,
            Self::Node { right, .. } => right.as_deref(),
        }
    }

    pub fn is_leaf(&self) -> bool {
        matches!(self, Self::Leaf(_))
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Leaf(s) => s.len(),
            Self::Node { len, .. } => *len,
        }
    }

    pub fn weight(&self) -> usize {
        match self {
            Self::Leaf(s) => s.len(),
            Self::Node { weight, .. } => *weight,
        }
    }

    pub fn depth(&self) -> usize {
        if self.is_leaf() {
            0
        } else {
            let left_depth = self.get_left().map_or(0, Self::depth);
            let right_depth = self.get_right().map_or(0, Self::depth);

            1 + std::cmp::max(left_depth, right_depth)
        }
    }

    pub fn is_balanced(&self) -> bool {
        let depth = self.depth();
        let weight = self.weight();

        if depth >= FIBONACCI_SEQUENCE.len() - 2 {
            return false;
        }

        FIBONACCI_SEQUENCE[depth + 2] <= weight
    }

    pub fn leaves(&self) -> impl Iterator<Item = &Self> {
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

    pub fn into_leaves(self) -> impl Iterator<Item = Self> {
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
            2 => {
                let first = std::mem::take(&mut leaves[0]);
                let second = std::mem::take(&mut leaves[1]);

                Self::node(Some(first), Some(second))
            }
            _ => {
                let mid = range / 2;
                let (left, right) = leaves.split_at_mut(mid);

                Self::node(Some(Self::merge(left)), Some(Self::merge(right)))
            }
        }
    }

    pub fn concat(self, other: Self) -> Self {
        Self::node(Some(self), Some(other)).rebalance()
    }

    pub fn char_at(&self, index: usize) -> Option<char> {
        match self {
            Self::Leaf(s) => s.chars().nth(index),
            Self::Node {
                left,
                right,
                weight,
                ..
            } => {
                if index < *weight {
                    left.as_deref()?.char_at(index)
                } else {
                    right.as_deref()?.char_at(index - weight)
                }
            }
        }
    }

    pub fn split(self, index: usize) -> (Option<Self>, Option<Self>) {
        match self {
            Self::Leaf(s) => {
                let (left_str, right_str) = s.split_at(index);
                let left_res = (!left_str.is_empty()).then(|| Self::leaf(left_str.to_string()));
                let right_res = (!right_str.is_empty()).then(|| Self::leaf(right_str.to_string()));

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

                if index < weight {
                    if let Some(l) = left_child {
                        let (split_left, split_right) = l.split(index);
                        (
                            split_left.map(Self::rebalance),
                            Some(Self::node(split_right, right_child).rebalance()),
                        )
                    } else {
                        (None, right_child)
                    }
                } else if index > weight {
                    if let Some(r) = right_child {
                        let (split_left, split_right) = r.split(index - weight);
                        (
                            Some(Self::node(left_child, split_left).rebalance()),
                            split_right.map(Self::rebalance),
                        )
                    } else {
                        (left_child, None)
                    }
                } else {
                    (left_child, right_child)
                }
            }
        }
    }

    pub fn insert(self, index: usize, text: String) -> Self {
        let (left_tree, right_tree) = self.split(index);
        let mid_tree = Self::leaf(text);

        let left_part = if let Some(l) = left_tree {
            l.concat(mid_tree)
        } else {
            mid_tree
        };

        if let Some(r) = right_tree {
            left_part.concat(r)
        } else {
            left_part
        }
    }

    pub fn remove(self, start: usize, len: usize) -> Self {
        let (lhs_l, lhs_r) = self.split(start);
        let rhs_r = lhs_r.and_then(|n| n.split(len).1);

        match (lhs_l, rhs_r) {
            (Some(l), Some(r)) => l.concat(r),
            (Some(l), None) => l,
            (None, Some(r)) => r,
            (None, None) => Self::default(),
        }
    }

    pub fn slice(&self, start: usize, len: usize) -> String {
        let mut result = String::with_capacity(len);
        let mut offset = 0;
        let end = start + len;

        for leaf in self.leaves() {
            if let Self::Leaf(s) = leaf {
                let leaf_len = s.len();

                let leaf_start = offset;
                let leaf_end = offset + leaf_len;

                if leaf_start < end && leaf_end > start {
                    let local_start = if start > leaf_start { start - leaf_start } else { 0 };
                    let local_end = if end < leaf_end { end - leaf_start } else { leaf_len };

                    result.push_str(&s[local_start..local_end]);
                }

                offset += leaf_len;
                if offset >= end {
                    break;
                }
            }
        }

        result
    }
}
