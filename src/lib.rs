// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! NStack
//!
//! A stack data structure with indexed lookup.
#![no_std]

mod annotation;
pub use annotation::*;

extern crate alloc;
use alloc::rc::Rc;

use core::mem;

const N: usize = 4;

#[allow(clippy::type_complexity)]
#[derive(Debug)]
pub enum NStack<T, A> {
    Leaf([Option<T>; N]),
    Node([Option<Annotated<Rc<NStack<T, A>>, A>>; N]),
}

impl<T, A> NStack<T, A> {
    /// Creates a new empty NStack
    pub fn new() -> Self {
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
    T: Clone,
    A: Default + Clone + Annotation<T>,
{
    /// Pushes a new element onto the stack
    pub fn push(&mut self, t: T) {
        match self._push(t) {
            Push::Ok => (),
            Push::NoRoom { t, .. } => {
                let old_root = mem::take(self);

                let mut new_node = [None, None, None, None];
                new_node[0] = Some(Annotated::new(Rc::new(old_root)));

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
                            match Rc::make_mut(&mut *anno.subtree_mut())
                                ._push(t)
                            {
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
                                                Some(Annotated::new(Rc::new(
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
                    node[index] = Some(Annotated::new(Rc::new(new_node)));
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
                        match Rc::make_mut(&mut *anno.subtree_mut())._pop() {
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

impl<T, A> Default for NStack<T, A>
where
    A: Annotation<NStack<T, A>>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, A> Clone for NStack<T, A>
where
    T: Clone,
    A: Clone,
{
    fn clone(&self) -> Self {
        match self {
            NStack::Leaf(anno) => NStack::Leaf(anno.clone()),
            NStack::Node(node) => NStack::Node(node.clone()),
        }
    }
}

impl<T, A> Annotation<NStack<T, A>> for A
where
    A: Default + Annotation<T>,
{
    fn from_subtree(stack: &NStack<T, A>) -> Self {
        match stack {
            NStack::Leaf(leaf) => {
                let mut anno = A::default();
                for t in leaf.iter().flatten() {
                    anno = anno.combine(&A::from_subtree(t));
                }
                anno
            }
            NStack::Node(node) => {
                let mut anno = A::default();
                for a in node.iter().flatten() {
                    anno = anno.combine(a.anno());
                }
                anno
            }
        }
    }
}
