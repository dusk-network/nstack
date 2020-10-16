// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! NStack
//!
//! A stack datastructure with indexed lookup.
#![warn(missing_docs)]
use core::mem;

use canonical::{Canon, Store};
use canonical_derive::Canon;

use microkelvin::{
    Annotated, Annotation, Branch, Cardinality, Compound, Traverse,
};

const N: usize = 4;

#[derive(Clone, Canon, Debug)]
pub enum NStack<T, A, S: Store> {
    Leaf([Option<T>; N]),
    Node([Option<Annotated<NStack<T, A, S>, A, S>>; N]),
}

impl<T, A, S> Compound for NStack<T, A, S>
where
    Self: Canon<S>,
    A: Annotation<T>,
    S: Store,
{
    type Leaf = T;
    type Annotation = A;

    fn annotation(&self) -> Self::Annotation {
        match self {
            NStack::Leaf([None, ..]) => A::identity(),
            NStack::Leaf([Some(a), None, ..]) => A::from_leaf(a),
            NStack::Leaf([Some(a), Some(b), None, ..]) => {
                A::op(&A::from_leaf(a), &A::from_leaf(b))
            }
            NStack::Leaf([Some(a), Some(b), Some(c), None]) => A::op(
                &A::op(&A::from_leaf(a), &A::from_leaf(b)),
                &A::from_leaf(c),
            ),
            NStack::Leaf([Some(a), Some(b), Some(c), Some(d)]) => A::op(
                &A::op(&A::from_leaf(a), &A::from_leaf(b)),
                &A::op(&A::from_leaf(c), &A::from_leaf(d)),
            ),
            NStack::Leaf(_) => unreachable!("Invalid leaf structure"),

            NStack::Node([None, ..]) => A::identity(),
            NStack::Node([Some(a), None, ..]) => a.annotation().clone(),
            NStack::Node([Some(a), Some(b), None, ..]) => {
                A::op(a.annotation(), b.annotation())
            }
            NStack::Node([Some(a), Some(b), Some(c), None]) => {
                A::op(&A::op(a.annotation(), b.annotation()), c.annotation())
            }
            NStack::Node([Some(a), Some(b), Some(c), Some(d)]) => A::op(
                &A::op(a.annotation(), b.annotation()),
                &A::op(c.annotation(), d.annotation()),
            ),
            NStack::Node(_) => unreachable!("Invalid node structure"),
        }
    }

    fn traverse<M: Annotation<<Self as Compound>::Leaf>>(
        &self,
        method: &mut M,
    ) -> Traverse {
        todo!()
    }
}

impl<T, A, S> Default for NStack<T, A, S>
where
    T: Canon<S>,
    A: Canon<S>,
    S: Store,
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

impl<T, A, S> NStack<T, A, S>
where
    T: Canon<S>,
    A: Canon<S> + Annotation<T>,
    S: Store,
{
    /// Creates a new empty NStack
    pub fn new() -> Self {
        Self::default()
    }

    /// Pushes a new element onto the stack
    pub fn push(&mut self, t: T) -> Result<(), S::Error> {
        match self._push(t)? {
            Push::Ok => Ok(()),
            Push::NoRoom { t, .. } => {
                let old_root = mem::take(self);

                let mut new_node = [None, None, None, None];
                new_node[0] = Some(Annotated::new(old_root)?);

                *self = NStack::Node(new_node);

                // the first child of our new root will be our old root
                self.push(t)
            }
        }
    }

    fn _push(&mut self, t: T) -> Result<Push<T>, S::Error> {
        match self {
            NStack::Leaf(leaf) => {
                for i in 0..N {
                    match leaf[i] {
                        ref mut empty @ None => {
                            *empty = Some(t);
                            return Ok(Push::Ok);
                        }
                        Some(_) => (),
                    }
                }
                return Ok(Push::NoRoom { t, depth: 0 });
            }
            NStack::Node(node) => {
                let mut insert_node = None;

                // find last node, searching from reverse
                for i in 0..N {
                    let i = N - i - 1;

                    match &mut node[i] {
                        None => (),
                        Some(annotated) => {
                            match annotated.val_mut()?._push(t)? {
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
                                                Some(Annotated::new(old_root)?),
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
                    node[index] = Some(Annotated::new(new_node)?);
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
    pub fn pop(&mut self) -> Result<Option<T>, S::Error> {
        match self._pop()? {
            Pop::Ok(t) | Pop::Last(t) => Ok(Some(t)),
            Pop::None => Ok(None),
        }
    }

    fn _pop(&mut self) -> Result<Pop<T>, S::Error> {
        let mut clear_node = None;

        match self {
            NStack::Leaf(leaf) => {
                for i in 0..N {
                    // reverse
                    let i = N - i - 1;
                    match leaf[i].take() {
                        Some(leaf) => {
                            if i > 0 {
                                return Ok(Pop::Ok(leaf));
                            } else {
                                return Ok(Pop::Last(leaf));
                            }
                        }
                        None => (),
                    }
                }
                Ok(Pop::None)
            }
            NStack::Node(node) => {
                for i in 0..N {
                    // reverse
                    let i = N - i - 1;
                    match node[i] {
                        Some(ref mut subtree) => {
                            match subtree.val_mut()?._pop()? {
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
                        None => (),
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

    fn get(&self, i: u64) -> Result<Option<Branch<Self, S>>, S::Error> {
        let mut search = Cardinality::new(i);
        Branch::traverse(self, &mut search)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use canonical_host::MemStore;
    use microkelvin::Cardinality;
    // use kelvin::quickcheck_stack;

    #[test]
    fn trivial() {
        let mut nt = NStack::<u32, Cardinality, MemStore>::new();
        assert_eq!(nt.pop().unwrap(), None);
    }

    #[test]
    fn push_pop() {
        let mut nt = NStack::<_, Cardinality, MemStore>::new();
        nt.push(8).unwrap();
        assert_eq!(nt.pop().unwrap(), Some(8));
    }

    #[test]
    fn double() {
        let mut nt = NStack::<_, Cardinality, MemStore>::new();
        nt.push(0).unwrap();
        nt.push(1).unwrap();
        assert_eq!(nt.pop().unwrap(), Some(1));
        assert_eq!(nt.pop().unwrap(), Some(0));
    }

    #[test]
    fn multiple() {
        let n = 1024;

        let mut nt = NStack::<_, Cardinality, MemStore>::new();

        for i in 0..n {
            nt.push(i).unwrap();
        }

        for i in 0..n {
            assert_eq!(nt.pop().unwrap(), Some(n - i - 1));
        }

        assert_eq!(nt.pop().unwrap(), None);
    }

    #[test]
    fn get() {
        let n = 128;

        let mut nstack = NStack::<_, Cardinality, MemStore>::new();

        for i in 0u64..n {
            println!("pushing {}", i);
            nstack.push(i).unwrap();

            for o in 0..i {
                assert_eq!(*nstack.get(o).unwrap().unwrap(), o);
            }

            assert!(nstack.get(i + 1).unwrap().is_none());
        }
    }

    // #[test]
    // fn get_mut() {
    //     let n = 1024;

    //     let mut nt = NStack::<_, Cardinality<u64>, MemStore>::new();

    //     for i in 0..n {
    //         nt.push(i).unwrap();
    //     }

    //     for i in 0..n {
    //         *nt.get_mut(i).unwrap().unwrap() += 1;
    //     }

    //     for i in 0..n {
    //         assert_eq!(*nt.get(i).unwrap().unwrap(), i + 1);
    //     }
    // }

    // // Assert that all branches are always of the same length
    // #[test]
    // fn branch_lengths() {
    //     let n = 256;

    //     let mut nt = NStack::<_, Cardinality<u64>, MemStore>::new();

    //     for i in 0..n {
    //         nt.push(i).unwrap();
    //     }

    //     let length_zero = nt.get(0).unwrap().unwrap().levels().len();

    //     for i in 1..n {
    //         assert_eq!(length_zero, nt.get(i).unwrap().unwrap().levels().len())
    //     }
    // }

    // quickcheck_stack!(|| NStack::<_, Cardinality<u64>, MemStore>::new());
}
