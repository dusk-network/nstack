// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use nstack::annotation::Cardinality;
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
        assert_eq!(Cardinality::from_child(&nt), (i + 1) as usize);
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

    let mut nt = NStack::<_, Cardinality>::new();

    for i in 0..n {
        nt.push(i);
    }

    let length_zero = nt.nth(0).expect("Some(_)").depth();

    for i in 1..n {
        assert_eq!(length_zero, nt.nth(i).expect("Some(_)").depth())
    }
}
