// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

extern crate alloc;

use core::ops::{Deref, DerefMut};

/// A subtree annotated with some metadata.
#[derive(Debug, Clone)]
pub struct Annotated<T, A> {
    subtree: T,
    anno: A,
}

impl<T, A> Annotated<T, A> {
    /// Returns the annotation over the subtree.
    pub fn anno(&self) -> &A {
        &self.anno
    }

    /// Returns the subtree.
    pub fn subtree(&self) -> &T {
        &self.subtree
    }
}

impl<T, A> Annotated<T, A>
where
    A: Annotation<T>,
{
    /// Create a new annotation over a subtree.
    pub fn new(subtree: T) -> Self {
        Self {
            anno: A::from_subtree(&subtree),
            subtree,
        }
    }

    /// Returns a mutable reference to the annotated subtree.
    pub fn subtree_mut(&mut self) -> AnnotatedRefMut<'_, T, A> {
        AnnotatedRefMut { anno: self }
    }
}

impl<T, A> Default for Annotated<T, A>
where
    T: Default,
    A: Annotation<T>,
{
    fn default() -> Self {
        let elem = T::default();
        Self::new(elem)
    }
}

impl<T, A> PartialEq for Annotated<T, A>
where
    T: PartialEq,
    A: Annotation<T>,
{
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self.subtree, &other.subtree)
    }
}

impl<T, A> Eq for Annotated<T, A>
where
    T: PartialEq + Eq,
    A: Annotation<T>,
{
}

impl<T, A> From<T> for Annotated<T, A>
where
    A: Annotation<T>,
{
    fn from(elem: T) -> Self {
        Self::new(elem)
    }
}

/// A mutable reference to an annotated subtree.
///
/// This allows for the mutation of the subtree, and subsequent re-computation
/// of the annotation when dropped.
pub struct AnnotatedRefMut<'a, T, A>
where
    A: Annotation<T>,
{
    anno: &'a mut Annotated<T, A>,
}

impl<'a, T, A> Drop for AnnotatedRefMut<'a, T, A>
where
    A: Annotation<T>,
{
    fn drop(&mut self) {
        let anno = A::from_subtree(&self.anno.subtree);
        self.anno.anno = anno;
    }
}

impl<'a, T, A> Deref for AnnotatedRefMut<'a, T, A>
where
    A: Annotation<T>,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.anno.subtree
    }
}

impl<'a, T, A> DerefMut for AnnotatedRefMut<'a, T, A>
where
    A: Annotation<T>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.anno.subtree
    }
}

pub trait Annotation<T> {
    fn from_subtree(t: &T) -> Self;
    fn combine(&self, other: &Self) -> Self;
}

// impl<T> Annotation<T> for () {
//     fn from_subtree(_: &T) -> Self {}
//
//     fn combine(&self, _: &Self) -> Self {}
// }
//
// impl<T, A> Annotation<Rc<T>> for A
// where
//     A: Annotation<T>,
// {
//     fn from_subtree(t: &Rc<T>) -> Self {
//         A::from_subtree(t.as_ref())
//     }
//
//     fn combine(&mut self, other: &Self) {
//         <A as Annotation<T>>::combine(self, other);
//     }
// }
