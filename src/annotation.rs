// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::NStack;

use core::ops::{AddAssign, Deref};

use ranno::Annotation;

/// The cardinality of the NStack.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Cardinality(usize);

impl Deref for Cardinality {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AddAssign<usize> for Cardinality {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl AddAssign for Cardinality {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
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
                    anno += 1;
                }
                anno
            }
            NStack::Node(node) => {
                let mut anno = Cardinality::default();
                for a in node.iter().flatten() {
                    anno += *a.anno();
                }
                anno
            }
        }
    }
}
