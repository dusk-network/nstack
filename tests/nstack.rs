// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use microkelvin::*;
use nstack::NStack;
use rkyv::{Archive, Deserialize};

#[test]
fn trivial() {
    let mut nt = NStack::<u32, Cardinality, OffsetLen>::new();
    assert_eq!(nt.pop(), None);
}

#[test]
fn push_pop() {
    let mut nt = NStack::<_, Cardinality, OffsetLen>::new();
    nt.push(8);
    assert_eq!(nt.pop(), Some(8));
}

#[test]
fn double() {
    let mut nt = NStack::<_, Cardinality, OffsetLen>::new();
    nt.push(0);
    nt.push(1);
    assert_eq!(nt.pop(), Some(1));
    assert_eq!(nt.pop(), Some(0));
}

#[test]
fn multiple() {
    let n = 1024;

    let mut nt = NStack::<_, Cardinality, OffsetLen>::new();

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

    let mut nstack = NStack::<_, Cardinality, OffsetLen>::new();

    for i in 0..n {
        nstack.push(i);
    }

    for i in 0..n {
        assert_eq!(nstack.walk(Nth(i)).expect("Some(_)").leaf(), i);
    }

    assert!(nstack.walk(Nth(n)).is_none());
}

#[test]
fn nth_mut() {
    let n: u64 = 1024;

    let mut nstack = NStack::<_, Cardinality, OffsetLen>::new();

    for i in 0..n {
        nstack.push(i);
    }

    for i in 0..n {
        let mut branch_mut = nstack.walk_mut(Nth(i)).expect("Some(_)");
        *branch_mut.leaf_mut() += 1;
    }

    for i in 0..n {
        assert_eq!(nstack.walk(Nth(i)).expect("Some(_)").leaf(), i + 1);
    }
}

// Assert that all branches are always of the same length
#[test]
fn branch_lengths() {
    let n = 256;

    let mut nt = NStack::<_, Cardinality, OffsetLen>::new();

    for i in 0..n {
        nt.push(i);
    }

    let length_reference = nt.walk(All).expect("Some(_)").depth();

    for i in 0..n {
        assert_eq!(length_reference, nt.walk(Nth(i)).expect("Some(_)").depth())
    }
}

#[test]
fn persist() {
    let n: u64 = 1024;

    let mut stack = NStack::<_, Cardinality, OffsetLen>::new();
    for i in 0..n {
        stack.push(i);
    }

    let store = StoreRef::new(HostStore::new());
    let stored = store.store(&stack);

    let restored: &<NStack<u64, Cardinality, OffsetLen> as Archive>::Archived =
        store.get::<NStack<_, Cardinality, OffsetLen>>(stored.ident());
    let restored: NStack<u64, Cardinality, OffsetLen> =
        restored.deserialize(&mut store.clone()).unwrap();

    // empty original
    for i in 0..n {
        assert_eq!(stack.pop().unwrap(), n - i - 1);
    }

    // check restored copy
    for i in 0..n {
        assert_eq!(restored.walk(Nth(i)).unwrap().leaf(), i);
    }
}
