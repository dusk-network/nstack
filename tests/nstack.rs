// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use nstack::NStack;

#[test]
fn trivial() {
    let mut nt = NStack::<u32>::new();
    assert_eq!(nt.pop(), None);
}

#[test]
fn push_pop() {
    let mut nt = NStack::new();
    nt.push(8);
    assert_eq!(nt.pop(), Some(8));
}

#[test]
fn double() {
    let mut nt = NStack::new();
    nt.push(0);
    nt.push(1);
    assert_eq!(nt.pop(), Some(1));
    assert_eq!(nt.pop(), Some(0));
}

#[test]
fn multiple() {
    let n = 1024;

    let mut nt = NStack::new();

    for i in 0..n {
        nt.push(i);
    }

    for i in 0..n {
        assert_eq!(nt.pop(), Some(n - i - 1));
    }

    assert_eq!(nt.pop(), None);
}
