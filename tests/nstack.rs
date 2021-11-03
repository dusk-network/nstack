// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use microkelvin::*;
use nstack::NStack;

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

    let mut nstack = NStack::<_, Cardinality>::new();

    for i in 0..n {
        nstack.push(i);
    }

    for i in 0..n {
        *nstack.walk_mut(Nth(i)).expect("Some(_)") += 1;
    }

    for i in 0..n {
        assert_eq!(nstack.walk(Nth(i)).expect("Some(_)").leaf(), i + 1);
    }
}

// Assert that all branches are always of the same length
#[test]
fn branch_lengths() {
    let n = 256;

    let mut nt = NStack::<_, Cardinality>::new();

    for i in 0..n {
        nt.push(i);
    }

    let length_reference = nt.walk(First).expect("Some(_)").depth();

    for i in 0..n {
        assert_eq!(length_reference, nt.walk(Nth(i)).expect("Some(_)").depth())
    }
}

#[test]
fn persist() {
    let n: u64 = 1024;

    let mut stack = NStack::<_, Cardinality>::new();
    for i in 0..n {
        stack.push(i);
    }

    let stored = Portal::put(&stack);
    let restored = Portal::get(stored);

    // empty original
    for i in 0..n {
        assert_eq!(stack.pop().unwrap(), n - i - 1);
    }

    // empty restored copy
    for i in 0..n {
        assert_eq!(restored.walk(Nth(i)).unwrap().leaf(), i);
    }
}
