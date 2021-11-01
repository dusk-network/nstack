// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use core::borrow::Borrow;

use microkelvin::*;
use nstack::NStack;
use rkyv::{rend::LittleEndian, Archive};

#[test]
fn trivial() {
    let mut nt = NStack::<u32, Cardinality>::new();
    assert_eq!(nt.pop(), None);
}

#[test]
fn push_pop() {
    let mut nt = NStack::<_, Cardinality>::new();
    nt.push(8);
    assert_eq!(nt.pop(), Some(8));
}

#[test]
fn double() {
    let mut nt = NStack::<_, Cardinality>::new();
    nt.push(0);
    nt.push(1);
    assert_eq!(nt.pop(), Some(1));
    assert_eq!(nt.pop(), Some(0));
}

#[test]
fn multiple() {
    let n = 1024;

    let mut nt = NStack::<_, Cardinality>::new();

    for i in 0..n {
        nt.push(i);
    }

    for i in 0..n {
        assert_eq!(nt.pop(), Some(n - i - 1));
    }

    assert_eq!(nt.pop(), None);
}

#[test]
fn nth() {
    let n: u64 = 1024;

    let mut nstack = NStack::<_, Cardinality>::new();

    for i in 0..n {
        let le: LittleEndian<u64> = i.into();
        nstack.push(le);
    }

    for i in 0..n {
        assert_eq!(*nstack.walk(Nth(i)).expect("Some(_)").leaf(), i);
    }

    assert!(nstack.walk(Nth(n)).is_none());
}

#[test]
fn nth_mut() {
    let n: u64 = 1024;

    let mut nstack = NStack::<_, Cardinality>::new();

    for i in 0..n {
        let le: LittleEndian<u64> = i.into();
        nstack.push(le);
    }

    for i in 0..n {
        *nstack.walk_mut(Nth(i)).expect("Some(_)") += 1;
    }

    for i in 0..n {
        assert_eq!(*nstack.walk(Nth(i)).expect("Some(_)").leaf(), i + 1);
    }
}

// Assert that all branches are always of the same length
#[test]
fn branch_lengths() {
    let n = 256;

    let mut nt = NStack::<_, Cardinality>::new();

    for i in 0..n {
        let le: LittleEndian<u64> = i.into();
        nt.push(le);
    }

    let length_reference = nt.walk(First).expect("Some(_)").depth();

    for i in 0..n {
        assert_eq!(length_reference, nt.walk(Nth(i)).expect("Some(_)").depth())
    }
}

#[derive(Clone, Debug, Archive)]
#[archive(as = "Self")]
#[archive(bound(archive = "
  K: Primitive,
  MaxKey<K>: Primitive,
"))]
struct MaxAndCardinality<K> {
    cardinality: Cardinality,
    max: MaxKey<K>,
}

impl<K> Default for MaxAndCardinality<K> {
    fn default() -> Self {
        Self {
            max: Default::default(),
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
        &self.max
    }
}

impl<K, L> Annotation<L> for MaxAndCardinality<K>
where
    L: Keyed<K>,
    K: Primitive + Clone + Ord,
{
    fn from_leaf(leaf: &L) -> Self {
        Self {
            cardinality: Cardinality::from_leaf(leaf),
            max: MaxKey::from_leaf(leaf),
        }
    }
}

impl<A, K> Combine<A> for MaxAndCardinality<K>
where
    A: Borrow<Cardinality> + Borrow<MaxKey<K>>,
    K: Clone + Ord,
{
    fn combine<C>(anno: AnnoIter<C, A>) -> Self
    where
        C: Compound<A> + Archive,
        C::Archived: ArchivedCompound<C, A>,
        C::Leaf: Archive,
        A: Annotation<C::Leaf>,
    {
        Self {
            cardinality: Cardinality::combine(anno.clone()),
            max: MaxKey::combine(anno),
        }
    }
}
