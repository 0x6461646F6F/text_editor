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

pub enum Rope {
    Leaf(String),
    Node {
        left: Option<Box<Rope>>,
        right: Option<Box<Rope>>,
        weight: usize,
    },
}

impl Rope {
    pub fn leaf(s: String) -> Self {
        Self::Leaf(s)
    }

    pub fn node(left: Option<Rope>, right: Option<Rope>) -> Self {
        Self::Node {
            weight: left.as_ref().map_or(0, |n| n.len()),
            left: left.map(|n| Box::new(n)),
            right: right.map(|n| Box::new(n)),
        }
    }

    pub fn getLeft(&self) -> Option<&Self> {
        match self {
            Rope::Leaf(_) => None,
            Rope::Node { left, .. } => left.as_deref(),
        }
    }

    pub fn getRight(&self) -> Option<&Self> {
        match self {
            Rope::Leaf(_) => None,
            Rope::Node { right, .. } => right.as_deref(),
        }
    }

    pub fn is_leaf(&self) -> bool {
        matches!(self, Rope::Leaf(_))
    }

    pub fn len(&self) -> usize {
        match self {
            Rope::Leaf(s) => s.len(),
            Rope::Node { right, weight, .. } => weight + right.as_deref().map_or(0, |n| n.len()),
        }
    }

    pub fn weight(&self) -> usize {
        match self {
            Rope::Leaf(s) => s.len(),
            Rope::Node { weight, .. } => *weight,
        }
    }

    pub fn depth(&self) -> usize {
        if self.is_leaf() {
            0
        } else {
            let left_depth = self.getLeft().map_or(0, |node| node.depth());
            let right_depth = self.getRight().map_or(0, |node| node.depth());

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
            c = node.getLeft();
        }

        std::iter::from_fn(move || {
            while let Some(node) = stack.pop() {
                if node.is_leaf() {
                    return Some(node);
                }

                let mut curr = node.getRight();
                while let Some(child) = curr {
                    stack.push(child);
                    curr = child.getLeft();
                }
            }

            None
        })
    }

    // fn merge(self, start: usize, end: usize) -> Self {
    //     todo!()
    // }

    // pub fn rebalance(self) -> Self {
    //     if !self.is_balanced() {
    //         let leaves: &[Rope] = self.leaves().collect();
    //         return leaves.merge;
    //     }

    //     self
    // }
}
