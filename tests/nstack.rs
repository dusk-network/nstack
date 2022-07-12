// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use nstack::{Annotation, NStack};

#[derive(Debug, Default, Clone)]
struct Cardinality(usize);

impl Annotation<u32> for Cardinality {
    fn from_subtree(_: &u32) -> Self {
        Cardinality(1)
    }

    fn combine(&self, other: &Self) -> Self {
        Self(self.0 + other.0)
    }
}

#[test]
fn trivial() {
    let mut nt = NStack::<u32, Cardinality>::new();
    assert_eq!(nt.pop(), None);
}

#[test]
fn push_pop() {
    let mut nt = NStack::<u32, Cardinality>::new();
    nt.push(8);
    assert_eq!(nt.pop(), Some(8));
}

#[test]
fn double() {
    let mut nt = NStack::<u32, Cardinality>::new();
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
        assert_eq!(Cardinality::from_subtree(&nt).0, (i + 1) as usize);
    }

    for i in 0..n {
        assert_eq!(nt.pop(), Some(n - i - 1));
    }

    assert_eq!(nt.pop(), None);
}
