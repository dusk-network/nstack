// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::annotation::Cardinality;
use crate::NStack;

use core::borrow::Borrow;

use microkelvin::{Branch, BranchMut, Child, Step, Walk, Walker};
use ranno::Annotation;

impl<T, A> NStack<T, A>
where
    A: Annotation<Self> + Borrow<Cardinality>,
{
    /// Construct a [`Branch`] pointing to the `nth` element, if any
    pub fn nth(&self, index: u64) -> Option<Branch<Self, A>> {
        Branch::walk(self, Index(index))
    }

    /// Construct a [`BranchMut`] pointing to the `nth` element, if any
    pub fn nth_mut(&mut self, index: u64) -> Option<BranchMut<Self, A>> {
        BranchMut::walk(self, Index(index))
    }
}

struct Index(u64);

impl<T, A> Walker<NStack<T, A>, A> for Index
where
    A: Annotation<NStack<T, A>> + Borrow<Cardinality>,
{
    fn walk(&mut self, walk: Walk<NStack<T, A>, A>) -> Step {
        for i in 0.. {
            match walk.child(i) {
                Child::Leaf(_) => {
                    if self.0 == 0 {
                        return Step::Found(i);
                    } else {
                        self.0 -= 1
                    }
                }
                Child::Node(node) => {
                    let anno = node.anno();
                    let c = (*anno).borrow();

                    let c = **c;

                    if self.0 < c {
                        return Step::Into(i);
                    }

                    self.0 -= c;
                }
                Child::Empty => (),
                Child::EndOfNode => return Step::Abort,
            }
        }
        unreachable!()
    }
}
