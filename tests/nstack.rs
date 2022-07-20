// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use core::borrow::Borrow;

use nstack::annotation::{Cardinality, Keyed, MaxKey};
use nstack::NStack;
use ranno::Annotation;

#[test]
fn trivial() {
    let mut nt = NStack::<u32, ()>::new();
    assert_eq!(nt.pop(), None);
}

#[test]
fn push_pop() {
    let mut nt = NStack::<u32, ()>::new();
    nt.push(8);
    assert_eq!(nt.pop(), Some(8));
}

#[test]
fn double() {
    let mut nt = NStack::<u32, ()>::new();
    nt.push(0);
    nt.push(1);
    assert_eq!(nt.pop(), Some(1));
    assert_eq!(nt.pop(), Some(0));
}

#[test]
fn multiple() {
    let n = 1024;

    let mut nt = NStack::<u32, Cardinality>::new();

    for i in 0..n {
        nt.push(i);
        assert_eq!(Cardinality::from_child(&nt), (i + 1) as u64);
    }

    for i in 0..n {
        assert_eq!(nt.pop(), Some(n - i - 1));
    }

    assert_eq!(nt.pop(), None);
}

#[test]
fn nth() {
    let n = 1024;

    let mut nstack = NStack::<_, Cardinality>::new();

    for i in 0..n {
        nstack.push(i);
    }

    for i in 0..n {
        assert_eq!(*nstack.nth(i).expect("Some(_)"), i);
    }

    assert!(nstack.nth(n).is_none());
}

#[test]
fn nth_mut() {
    let n = 1024;

    let mut nstack = NStack::<_, Cardinality>::new();

    for i in 0..n {
        nstack.push(i);
    }

    for i in 0..n {
        *nstack.nth_mut(i).expect("Some(_)") += 1;
    }

    for i in 0..n {
        assert_eq!(*nstack.nth(i).expect("Some(_)"), i + 1);
    }
}

#[test]
fn branch_lengths() {
    let n = 256;

    let mut nt = NStack::<_, MaxAndCardinality<u64>>::new();

    for i in 0..n {
        nt.push(i);
    }

    let length_zero = nt.nth(0).expect("Some(_)").depth();

    for i in 1..n {
        assert_eq!(length_zero, nt.nth(i).expect("Some(_)").depth())
    }

    let max_branch = nt.max_key().expect("Should have a max");
    assert_eq!(*max_branch, n - 1);
}

#[derive(Debug, Clone)]
struct MaxAndCardinality<K> {
    cardinality: Cardinality,
    max_key: MaxKey<K>,
}

impl<T, K> Annotation<NStack<T, MaxAndCardinality<K>>> for MaxAndCardinality<K>
where
    T: Keyed<K>,
    K: Clone + PartialOrd,
{
    fn from_child(stack: &NStack<T, MaxAndCardinality<K>>) -> Self {
        let mut max_key = MaxKey::<K>::NegativeInfinity;
        let mut cardinality = 0;

        match stack {
            NStack::Leaf(leaf) => {
                for key in leaf.iter().flatten().map(Keyed::key) {
                    if &max_key < key {
                        max_key = MaxKey::Maximum(key.clone());
                    }
                    cardinality += 1;
                }
            }
            NStack::Node(node) => {
                for annotated in node.iter().flatten() {
                    let anno = &*annotated.anno();
                    if max_key < anno.max_key {
                        max_key = anno.max_key.clone();
                    }
                    cardinality += *anno.cardinality;
                }
            }
        }

        Self {
            cardinality: cardinality.into(),
            max_key,
        }
    }
}

impl<K> Default for MaxAndCardinality<K> {
    fn default() -> Self {
        Self {
            max_key: Default::default(),
            cardinality: Default::default(),
        }
    }
}

impl<K> Borrow<Cardinality> for MaxAndCardinality<K> {
    fn borrow(&self) -> &Cardinality {
        &self.cardinality
    }
}

impl<K> Borrow<MaxKey<K>> for MaxAndCardinality<K> {
    fn borrow(&self) -> &MaxKey<K> {
        &self.max_key
    }
}
