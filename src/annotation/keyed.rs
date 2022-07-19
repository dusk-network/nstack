// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::NStack;

use core::borrow::Borrow;
use core::cmp::{Ordering, PartialOrd};
use core::marker::PhantomData;

use microkelvin::{Branch, BranchMut, Child, Step, Walk, Walker};
use ranno::Annotation;

impl<T, A> NStack<T, A>
where
    A: Annotation<NStack<T, A>>,
{
    /// Construct a [`Branch`] pointing to the element with the largest key
    pub fn max_key<K>(&self) -> Option<Branch<Self, A>>
    where
        T: Keyed<K>,
        A: Borrow<MaxKey<K>>,
        K: Clone + PartialOrd,
    {
        // Return the first that satisfies the walk
        Branch::walk(self, FindMaxKey::<K>::default())
    }

    /// Construct a [`BranchMut`] pointing to the element with the largest key
    pub fn max_key_mut<K>(&mut self) -> Option<BranchMut<Self, A>>
    where
        T: Keyed<K>,
        A: Borrow<MaxKey<K>>,
        K: Clone + PartialOrd,
    {
        // Return the first mutable branch that satisfies the walk
        BranchMut::walk(self, FindMaxKey::<K>::default())
    }
}

/// Trait for getting the key from a Leaf value
pub trait Keyed<K> {
    /// Return a reference to the key of the leaf type
    fn key(&self) -> &K;
}

impl<K> Keyed<K> for K {
    fn key(&self) -> &K {
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaxKey<K> {
    /// Every other key is larger
    NegativeInfinity,
    /// Actual max key
    Maximum(K),
}

impl<K> Default for MaxKey<K> {
    fn default() -> Self {
        Self::NegativeInfinity
    }
}

impl<K> PartialEq<K> for MaxKey<K>
where
    K: PartialEq,
{
    fn eq(&self, other: &K) -> bool {
        match self {
            MaxKey::NegativeInfinity => false,
            MaxKey::Maximum(key) => key.eq(other),
        }
    }
}

impl<K> PartialOrd for MaxKey<K>
where
    K: PartialOrd,
{
    fn partial_cmp(&self, other: &MaxKey<K>) -> Option<Ordering> {
        match other {
            MaxKey::NegativeInfinity => Some(Ordering::Greater),
            MaxKey::Maximum(other) => self.partial_cmp(other),
        }
    }
}

impl<K> PartialOrd<K> for MaxKey<K>
where
    K: PartialOrd,
{
    fn partial_cmp(&self, other: &K) -> Option<Ordering> {
        match self {
            MaxKey::NegativeInfinity => Some(Ordering::Less),
            MaxKey::Maximum(key) => key.partial_cmp(other),
        }
    }
}

impl<T, K> Annotation<NStack<T, MaxKey<K>>> for MaxKey<K>
where
    T: Keyed<K>,
    K: Clone + PartialOrd,
{
    fn from_child(stack: &NStack<T, MaxKey<K>>) -> Self {
        let mut max_key = Self::NegativeInfinity;

        match stack {
            NStack::Leaf(leaf) => {
                for key in leaf.iter().flatten().map(Keyed::key) {
                    if &max_key < key {
                        max_key = MaxKey::Maximum(key.clone());
                    }
                }
            }
            NStack::Node(node) => {
                for annotated in node.iter().flatten() {
                    let key = &*annotated.anno();
                    if &max_key < key {
                        max_key = key.clone();
                    }
                }
            }
        }

        max_key
    }
}

struct FindMaxKey<K>(PhantomData<K>);

impl<K> Default for FindMaxKey<K> {
    fn default() -> Self {
        FindMaxKey(PhantomData::default())
    }
}

impl<T, A, K> Walker<NStack<T, A>, A> for FindMaxKey<K>
where
    T: Keyed<K>,
    A: Annotation<NStack<T, A>> + Borrow<MaxKey<K>>,
    K: Clone + PartialOrd,
{
    fn walk(&mut self, walk: Walk<NStack<T, A>, A>) -> Step {
        let mut current_max = MaxKey::NegativeInfinity;
        let mut current_step = Step::Abort;

        for i in 0.. {
            match walk.child(i) {
                Child::Leaf(l) => {
                    let leaf_max = MaxKey::Maximum(l.key().clone());

                    if leaf_max > current_max {
                        current_max = leaf_max;
                        current_step = Step::Found(i);
                    }
                }
                Child::Node(n) => {
                    let anno = n.anno();
                    let node_max = (*anno).borrow();
                    if node_max > &current_max {
                        current_max = node_max.clone();
                        current_step = Step::Into(i);
                    }
                }
                Child::Empty => (),
                Child::EndOfNode => return current_step,
            }
        }
        unreachable!()
    }
}
