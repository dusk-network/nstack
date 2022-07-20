// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::NStack;

use core::ops::Deref;

use ranno::Annotation;

/// The cardinality of the NStack.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cardinality(u64);

impl From<u64> for Cardinality {
    fn from(c: u64) -> Self {
        Self(c)
    }
}

impl Deref for Cardinality {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq<u64> for Cardinality {
    fn eq(&self, other: &u64) -> bool {
        self.0.eq(other)
    }
}

impl<T> Annotation<NStack<T, Cardinality>> for Cardinality {
    fn from_child(stack: &NStack<T, Cardinality>) -> Self {
        let mut cardinality = 0;

        match stack {
            NStack::Leaf(leaf) => {
                for _ in leaf.iter().flatten() {
                    cardinality += 1;
                }
            }
            NStack::Node(node) => {
                for a in node.iter().flatten() {
                    let anno = a.anno();
                    let c = &*anno;
                    cardinality += c.0;
                }
            }
        }

        cardinality.into()
    }
}
