// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! NStack
//!
//! A stack datastructure with indexed lookup.
#![no_std]
#![allow(clippy::large_enum_variant)]
use core::mem;

use canonical::{Canon, CanonError};
use canonical_derive::Canon;

use microkelvin::{
    Annotation, Child, ChildMut, Compound, GenericChild, GenericTree, Link,
    MutableLeaves,
};

const N: usize = 4;

// Clippy complains about the difference in size between the enum variants, however since the
// most common case is the larger enum, and node traversal should be fast, we trade memory for
// speed here.
#[derive(Clone, Canon, Debug)]
pub enum NStack<T, A>
where
    Self: Compound<A>,
    A: Annotation<<Self as Compound<A>>::Leaf>,
    T: Canon,
{
    Leaf([Option<T>; N]),
    Node([Option<Link<NStack<T, A>, A>>; N]),
}

impl<T, A> Compound<A> for NStack<T, A>
where
    T: Canon,
    A: Canon + Annotation<T>,
{
    type Leaf = T;

    fn child(&self, ofs: usize) -> Child<Self, A> {
        match (ofs, self) {
            (0, NStack::Node([Some(a), _, _, _])) => Child::Node(a),
            (1, NStack::Node([_, Some(b), _, _])) => Child::Node(b),
            (2, NStack::Node([_, _, Some(c), _])) => Child::Node(c),
            (3, NStack::Node([_, _, _, Some(d)])) => Child::Node(d),
            (0, NStack::Leaf([Some(a), _, _, _])) => Child::Leaf(a),
            (1, NStack::Leaf([_, Some(b), _, _])) => Child::Leaf(b),
            (2, NStack::Leaf([_, _, Some(c), _])) => Child::Leaf(c),
            (3, NStack::Leaf([_, _, _, Some(d)])) => Child::Leaf(d),
            _ => Child::EndOfNode,
        }
    }

    fn child_mut(&mut self, ofs: usize) -> ChildMut<Self, A> {
        match (ofs, self) {
            (0, NStack::Node([Some(a), _, _, _])) => ChildMut::Node(a),
            (1, NStack::Node([_, Some(b), _, _])) => ChildMut::Node(b),
            (2, NStack::Node([_, _, Some(c), _])) => ChildMut::Node(c),
            (3, NStack::Node([_, _, _, Some(d)])) => ChildMut::Node(d),
            (0, NStack::Leaf([Some(a), _, _, _])) => ChildMut::Leaf(a),
            (1, NStack::Leaf([_, Some(b), _, _])) => ChildMut::Leaf(b),
            (2, NStack::Leaf([_, _, Some(c), _])) => ChildMut::Leaf(c),
            (3, NStack::Leaf([_, _, _, Some(d)])) => ChildMut::Leaf(d),
            _ => ChildMut::EndOfNode,
        }
    }

    fn from_generic(tree: &GenericTree) -> Result<Self, CanonError> {
        let mut child_iter = tree.children().iter();

        match child_iter.next() {
            // empty case
            None => Ok(NStack::default()),
            // Empty nodes are invalid in NStack
            Some(GenericChild::Empty) => Err(CanonError::InvalidEncoding),
            Some(GenericChild::Leaf(leaf)) => {
                let mut leaves = [Some(leaf.cast()?), None, None, None];
                for (i, child) in child_iter.enumerate() {
                    if let GenericChild::Leaf(leaf) = child {
                        leaves[i + 1] = Some(leaf.cast()?);
                    } else {
                        return Err(CanonError::InvalidEncoding);
                    }
                }
                Ok(NStack::Leaf(leaves))
            }
            Some(GenericChild::Link(id, anno)) => {
                let mut links = [
                    Some(Link::new_persisted(*id, anno.cast()?)),
                    None,
                    None,
                    None,
                ];
                for (i, child) in child_iter.enumerate() {
                    if let GenericChild::Link(id, anno) = child {
                        links[i + 1] =
                            Some(Link::new_persisted(*id, anno.cast()?));
                    } else {
                        return Err(CanonError::InvalidEncoding);
                    }
                }
                Ok(NStack::Node(links))
            }
        }
    }
}

impl<T, A> MutableLeaves for NStack<T, A>
where
    A: Annotation<T> + Canon,
    T: Canon,
{
}

impl<T, A> Default for NStack<T, A>
where
    A: Annotation<T> + Canon,
    T: Canon,
{
    fn default() -> Self {
        NStack::Leaf([None, None, None, None])
    }
}

enum Push<T> {
    Ok,
    NoRoom { t: T, depth: usize },
}

enum Pop<T> {
    Ok(T),
    Last(T),
    None,
}

impl<T, A> NStack<T, A>
where
    Self: Compound<A>,
    T: Canon,
    A: Canon + Annotation<<Self as Compound<A>>::Leaf> + Annotation<T>,
{
    /// Creates a new empty NStack
    pub fn new() -> Self {
        Self::default()
    }

    /// Pushes a new element onto the stack
    pub fn push(&mut self, t: T) -> Result<(), CanonError> {
        match self._push(t)? {
            Push::Ok => Ok(()),
            Push::NoRoom { t, .. } => {
                let old_root = mem::take(self);

                let mut new_node = [None, None, None, None];
                new_node[0] = Some(Link::new(old_root));

                *self = NStack::Node(new_node);

                // the first child of our new root will be our old root
                self.push(t)
            }
        }
    }

    fn _push(&mut self, t: T) -> Result<Push<T>, CanonError> {
        match self {
            NStack::Leaf(leaf) => {
                for mut item in leaf.iter_mut() {
                    match item {
                        ref mut empty @ None => {
                            **empty = Some(t);
                            return Ok(Push::Ok);
                        }
                        Some(_) => (),
                    }
                }
                Ok(Push::NoRoom { t, depth: 0 })
            }
            NStack::Node(node) => {
                let mut insert_node = None;

                // find last node, searching from reverse
                for i in 0..N {
                    let i = N - i - 1;

                    match &mut node[i] {
                        None => (),
                        Some(annotated) => {
                            match annotated.inner_mut()?._push(t)? {
                                Push::Ok => return Ok(Push::Ok),
                                Push::NoRoom { t, depth } => {
                                    // Are we in the last node
                                    if i == N - 1 {
                                        return Ok(Push::NoRoom {
                                            t,
                                            depth: depth + 1,
                                        });
                                    } else {
                                        // create a new node
                                        let mut new_node = NStack::Leaf([
                                            Some(t),
                                            None,
                                            None,
                                            None,
                                        ]);

                                        // give it enough depth
                                        for _ in 0..depth {
                                            let old_root = mem::replace(
                                                &mut new_node,
                                                NStack::new(),
                                            );
                                            new_node = NStack::Node([
                                                Some(Link::new(old_root)),
                                                None,
                                                None,
                                                None,
                                            ]);
                                        }

                                        // Insert node
                                        insert_node = Some((new_node, i + 1));
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                // break out and insert
                if let Some((new_node, index)) = insert_node {
                    node[index] = Some(Link::new(new_node));
                } else {
                    unreachable!()
                }

                Ok(Push::Ok)
            }
        }
    }

    /// Pop an element off the stack.
    ///
    /// Returns the popped element, if any.
    pub fn pop(&mut self) -> Result<Option<T>, CanonError> {
        match self._pop()? {
            Pop::Ok(t) | Pop::Last(t) => Ok(Some(t)),
            Pop::None => Ok(None),
        }
    }

    fn _pop(&mut self) -> Result<Pop<T>, CanonError> {
        let mut clear_node = None;

        match self {
            NStack::Leaf(leaf) => {
                for i in 0..N {
                    // reverse
                    let i = N - i - 1;
                    if let Some(leaf) = leaf[i].take() {
                        if i > 0 {
                            return Ok(Pop::Ok(leaf));
                        } else {
                            return Ok(Pop::Last(leaf));
                        }
                    }
                }
                Ok(Pop::None)
            }
            NStack::Node(node) => {
                for i in 0..N {
                    // reverse
                    let i = N - i - 1;
                    if let Some(ref mut subtree) = node[i] {
                        match subtree.inner_mut()?._pop()? {
                            Pop::Ok(t) => return Ok(Pop::Ok(t)),
                            Pop::Last(t) => {
                                if i == 0 {
                                    return Ok(Pop::Last(t));
                                } else {
                                    clear_node = Some((t, i));
                                    break;
                                }
                            }
                            Pop::None => return Ok(Pop::None),
                        }
                    }
                }
                if let Some((popped, clear_index)) = clear_node {
                    node[clear_index] = None;
                    Ok(Pop::Ok(popped))
                } else {
                    unreachable!()
                }
            }
        }
    }
}
