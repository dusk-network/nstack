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
pub struct Cardinality(usize);

impl From<usize> for Cardinality {
    fn from(c: usize) -> Self {
        Self(c)
    }
}

impl Deref for Cardinality {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq<usize> for Cardinality {
    fn eq(&self, other: &usize) -> bool {
        self.0.eq(other)
    }
}

impl<T> Annotation<NStack<T, Cardinality>> for Cardinality {
    fn from_child(stack: &NStack<T, Cardinality>) -> Self {
        match stack {
            NStack::Leaf(leaf) => {
                let mut anno = Cardinality::default();
                for _ in leaf.iter().flatten() {
                    anno.0 += 1;
                }
                anno
            }
            NStack::Node(node) => {
                let mut anno = Cardinality::default();
                for a in node.iter().flatten() {
                    let a = a.anno();
                    anno.0 += (*a).0;
                }
                anno
            }
        }
    }
}
