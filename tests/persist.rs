// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use nstack::NStack;

use std::io;

use rkyv::rend::LittleEndian;

use microkelvin::{ArchivedCompound, Cardinality, Nth, Portal};

fn persist() -> Result<(), io::Error> {
    let n: u64 = 1024;

    let mut stack = NStack::<_, Cardinality>::new();
    for i in 0..n {
        let i = LittleEndian::from(i);
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
        assert_eq!(*restored.walk(Nth(i)).unwrap().leaf(), i);
    }

    Ok(())
}

#[test]
fn persist_a() -> Result<(), io::Error> {
    persist()
}

#[test]
fn persist_b() -> Result<(), io::Error> {
    persist()
}

#[test]
fn persist_c() -> Result<(), io::Error> {
    persist()
}

#[test]
fn persist_d() -> Result<(), io::Error> {
    persist()
}

fn persist_across_threads() -> Result<(), io::Error> {
    let n: u64 = 1024;

    let mut stack = NStack::<_, Cardinality>::new();
    for i in 0..n {
        let i = LittleEndian::from(i);
        stack.push(i);
    }

    let stored = Portal::put(&stack);

    std::thread::spawn(move || {
        let restored = Portal::get(stored);
        for i in 0..n {
            assert_eq!(*restored.walk(Nth(i)).unwrap().leaf(), i);
        }
    })
    .join()
    .expect("thread to join cleanly");

    // empty original
    for i in 0..n {
        assert_eq!(stack.pop().unwrap(), n - i - 1);
    }

    Ok(())
}

#[test]
fn persist_across_threads_a() -> Result<(), io::Error> {
    persist_across_threads()
}

#[test]
fn persist_across_threads_b() -> Result<(), io::Error> {
    persist_across_threads()
}

#[test]
fn persist_across_threads_c() -> Result<(), io::Error> {
    persist_across_threads()
}

#[test]
fn persist_across_threads_d() -> Result<(), io::Error> {
    persist_across_threads()
}
