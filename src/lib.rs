// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! NStack
//!
//! A stack data structure with indexed lookup.
#![no_std]

pub mod annotation;

extern crate alloc;
use alloc::boxed::Box;

use core::mem;

use microkelvin::{Child, ChildMut, Compound, MutableLeaves};
use ranno::{Annotated, Annotation};

const N: usize = 4;

type NStackRef<T, A> = Box<NStack<T, A>>;

#[derive(Debug)]
pub enum NStack<T, A> {
    Leaf([Option<T>; N]),
    Node([Option<Annotated<NStackRef<T, A>, A>>; N]),
}

impl<T, A> NStack<T, A> {
    /// Creates a new empty NStack
    pub const fn new() -> Self {
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

impl<T, A> Compound<A> for NStack<T, A> {
    type Leaf = T;

    fn child(&self, index: usize) -> Child<Self, A> {
        match (index, self) {
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

    fn child_mut(&mut self, index: usize) -> ChildMut<Self, A> {
        match (index, self) {
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
}

impl<T, A> NStack<T, A>
where
    A: Annotation<Self>,
{
    /// Pushes a new element onto the stack
    pub fn push(&mut self, t: T) {
        match self._push(t) {
            Push::Ok => (),
            Push::NoRoom { t, .. } => {
                let old_root = mem::take(self);

                let mut new_node = [None, None, None, None];
                new_node[0] = Some(Annotated::new(Box::new(old_root)));

                *self = NStack::Node(new_node);

                // the first child of our new root will be our old root
                self.push(t)
            }
        }
    }

    fn _push(&mut self, t: T) -> Push<T> {
        match self {
            NStack::Leaf(leaf) => {
                for mut item in leaf.iter_mut() {
                    match item {
                        ref mut empty @ None => {
                            **empty = Some(t);
                            return Push::Ok;
                        }
                        Some(_) => (),
                    }
                }
                Push::NoRoom { t, depth: 0 }
            }
            NStack::Node(node) => {
                let mut insert_node = None;

                // find last node, searching from reverse
                for i in 0..N {
                    let i = N - i - 1;

                    match &mut node[i] {
                        None => (),
                        Some(anno) => {
                            match anno.child_mut()._push(t) {
                                Push::Ok => return Push::Ok,
                                Push::NoRoom { t, depth } => {
                                    // Are we in the last node
                                    if i == N - 1 {
                                        return Push::NoRoom {
                                            t,
                                            depth: depth + 1,
                                        };
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
                                                Some(Annotated::new(Box::new(
                                                    old_root,
                                                ))),
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
                    node[index] = Some(Annotated::new(Box::new(new_node)));
                } else {
                    unreachable!()
                }

                Push::Ok
            }
        }
    }

    /// Pop an element off the stack.
    ///
    /// Returns the popped element, if any.
    pub fn pop(&mut self) -> Option<T> {
        match self._pop() {
            Pop::Ok(t) | Pop::Last(t) => Some(t),
            Pop::None => None,
        }
    }

    fn _pop(&mut self) -> Pop<T> {
        let mut clear_node = None;

        match self {
            NStack::Leaf(leaf) => {
                for i in 0..N {
                    // reverse
                    let i = N - i - 1;
                    if let Some(leaf) = leaf[i].take() {
                        return if i > 0 {
                            Pop::Ok(leaf)
                        } else {
                            Pop::Last(leaf)
                        };
                    }
                }
                Pop::None
            }
            NStack::Node(node) => {
                for i in 0..N {
                    // reverse
                    let i = N - i - 1;
                    if let Some(ref mut anno) = node[i] {
                        match anno.child_mut()._pop() {
                            Pop::Ok(t) => return Pop::Ok(t),
                            Pop::Last(t) => {
                                if i == 0 {
                                    return Pop::Last(t);
                                } else {
                                    clear_node = Some((t, i));
                                    break;
                                }
                            }
                            Pop::None => return Pop::None,
                        }
                    }
                }
                if let Some((popped, clear_index)) = clear_node {
                    node[clear_index] = None;
                    Pop::Ok(popped)
                } else {
                    unreachable!()
                }
            }
        }
    }
}

impl<T, A> MutableLeaves for NStack<T, A> {}

impl<T, A> Default for NStack<T, A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, A> Clone for NStack<T, A>
where
    T: Clone,
    A: Annotation<Self>,
{
    fn clone(&self) -> Self {
        match self {
            NStack::Leaf(anno) => NStack::Leaf(anno.clone()),
            NStack::Node(node) => NStack::Node(node.clone()),
        }
    }
}
