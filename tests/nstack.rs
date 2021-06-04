// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use canonical::{Canon, CanonError};
use canonical_derive::Canon;

use nstack::NStack;

use core::borrow::Borrow;
use microkelvin::*;

#[test]
fn trivial() {
    let mut nt = NStack::<u32, Cardinality>::new();
    assert_eq!(nt.pop().unwrap(), None);
}

#[test]
fn push_pop() {
    let mut nt = NStack::<_, Cardinality>::new();
    nt.push(8).unwrap();
    assert_eq!(nt.pop().unwrap(), Some(8));
}

#[test]
fn double() {
    let mut nt = NStack::<_, Cardinality>::new();
    nt.push(0).unwrap();
    nt.push(1).unwrap();
    assert_eq!(nt.pop().unwrap(), Some(1));
    assert_eq!(nt.pop().unwrap(), Some(0));
}

#[test]
fn multiple() {
    let n = 1024;

    let mut nt = NStack::<_, Cardinality>::new();

    for i in 0..n {
        nt.push(i).unwrap();
    }

    for i in 0..n {
        assert_eq!(nt.pop().unwrap(), Some(n - i - 1));
    }

    assert_eq!(nt.pop().unwrap(), None);
}

#[test]
fn nth() {
    let n: u64 = 1024;

    let mut nstack = NStack::<_, Cardinality>::new();

    for i in 0..n {
        nstack.push(i).unwrap();
    }

    for i in 0..n {
        assert_eq!(*nstack.nth(i).unwrap().unwrap(), i);
    }

    assert!(nstack.nth(n).unwrap().is_none());
}

#[test]
fn nth_mut() -> Result<(), CanonError> {
    let n: u64 = 1024;

    let mut nstack = NStack::<_, Cardinality>::new();

    for i in 0..n {
        nstack.push(i)?;
    }

    for i in 0..n {
        *nstack.nth_mut(i)?.unwrap() += 1;
    }

    for i in 0..n {
        assert_eq!(*nstack.nth(i)?.unwrap(), i + 1);
    }

    Ok(())
}

// Assert that all branches are always of the same length
#[test]
fn branch_lengths() -> Result<(), CanonError> {
    let n = 256;

    let mut nt = NStack::<_, Cardinality>::new();

    for i in 0..n {
        nt.push(i)?;
    }

    let length_zero = nt.nth(0)?.unwrap().depth();

    for i in 1..n {
        assert_eq!(length_zero, nt.nth(i)?.unwrap().depth())
    }

    Ok(())
}

#[derive(Canon, Clone, Debug)]
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
    K: Clone + Ord + Canon,
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
        C: Compound<A>,
        A: Annotation<C::Leaf>,
    {
        Self {
            cardinality: Cardinality::combine(anno.clone()),
            max: MaxKey::combine(anno),
        }
    }
}
