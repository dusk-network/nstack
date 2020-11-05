// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! NStack
//!
//! A stack datastructure with indexed lookup.
#[cfg(feature = "host")]
use core::mem;

use canonical::{Canon, Store};
use canonical_derive::Canon;

use microkelvin::{Annotated, Annotation, Child, ChildMut, Compound};

const N: usize = 4;

#[derive(Clone, Canon, Debug)]
pub enum NStack<T, A, S>
where
    A: Canon<S>,
    T: Canon<S>,
    S: Store,
{
    Leaf([Option<T>; N]),
    Node([Option<Annotated<NStack<T, A, S>, S>>; N]),
}

impl<T, A, S> Compound<S> for NStack<T, A, S>
where
    T: Canon<S>,
    A: Canon<S>,
    S: Store,
{
    type Leaf = T;
    type Annotation = A;

    fn child(&self, ofs: usize) -> Child<Self, S> {
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

    fn child_mut(&mut self, ofs: usize) -> ChildMut<Self, S> {
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
}

impl<T, A, S> Default for NStack<T, A, S>
where
    T: Canon<S>,
    A: Canon<S> + Annotation<Self, S>,
    S: Store,
{
    fn default() -> Self {
        NStack::Leaf([None, None, None, None])
    }
}

#[cfg(feature = "host")]
enum Push<T> {
    Ok,
    NoRoom { t: T, depth: usize },
}

#[cfg(feature = "host")]
enum Pop<T> {
    Ok(T),
    Last(T),
    None,
}

impl<T, A, S> NStack<T, A, S>
where
    T: Canon<S>,
    A: Canon<S> + Annotation<Self, S>,
    S: Store,
{
    /// Creates a new empty NStack
    pub fn new() -> Self {
        Self::default()
    }

    /// Pushes a new element onto the stack
    #[cfg(feature = "host")]
    pub fn push(&mut self, t: T) -> Result<(), S::Error> {
        match self._push(t)? {
            Push::Ok => Ok(()),
            Push::NoRoom { t, .. } => {
                let old_root = mem::take(self);

                let mut new_node = [None, None, None, None];
                new_node[0] = Some(Annotated::new(old_root));

                *self = NStack::Node(new_node);

                // the first child of our new root will be our old root
                self.push(t)
            }
        }
    }

    #[cfg(feature = "host")]
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
                                                Some(Annotated::new(old_root)),
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
                    node[index] = Some(Annotated::new(new_node));
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
    #[cfg(feature = "host")]
    pub fn pop(&mut self) -> Result<Option<T>, S::Error> {
        match self._pop()? {
            Pop::Ok(t) | Pop::Last(t) => Ok(Some(t)),
            Pop::None => Ok(None),
        }
    }

    #[cfg(feature = "host")]
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
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::borrow::Borrow;

    use canonical::Canon;
    use canonical_host::MemStore;
    use microkelvin::{Annotation, Cardinality, Max, Nth};

    // Brach len
    const BLEN: usize = 5;

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
    fn nth() {
        let n: u64 = 1024;

        let mut nstack = NStack::<_, Cardinality, MemStore>::new();

        for i in 0..n {
            nstack.push(i).unwrap();
        }

        for i in 0..n {
            assert_eq!(*nstack.nth::<BLEN>(i).unwrap().unwrap(), i);
        }

        assert!(nstack.nth::<BLEN>(n).unwrap().is_none());
    }

    #[test]
    fn nth_mut() -> Result<(), <MemStore as Store>::Error> {
        let n: u64 = 1024;

        let mut nstack = NStack::<_, Cardinality, MemStore>::new();

        for i in 0..n {
            nstack.push(i)?;
        }

        for i in 0..n {
            *nstack.nth_mut::<BLEN>(i)?.unwrap() += 1;
        }

        for i in 0..n {
            assert_eq!(*nstack.nth::<BLEN>(i)?.unwrap(), i + 1);
        }

        Ok(())
    }

    // Assert that all branches are always of the same length
    #[test]
    fn branch_lengths() -> Result<(), <MemStore as Store>::Error> {
        let n = 256;

        let mut nt = NStack::<_, Cardinality, MemStore>::new();

        for i in 0..n {
            nt.push(i)?;
        }

        let length_zero = nt.nth::<BLEN>(0)?.unwrap().depth();

        for i in 1..n {
            assert_eq!(length_zero, nt.nth::<BLEN>(i)?.unwrap().depth())
        }

        Ok(())
    }

    #[derive(Canon, Clone, Debug)]
    struct MaxAndCardinality {
        cardinality: Cardinality,
        max: Max<u32>,
    }

    impl Borrow<Cardinality> for MaxAndCardinality {
        fn borrow(&self) -> &Cardinality {
            &self.cardinality
        }
    }

    impl Borrow<Max<u32>> for MaxAndCardinality {
        fn borrow(&self) -> &Max<u32> {
            &self.max
        }
    }

    impl Borrow<u32> for MaxAndCardinality {
        fn borrow(&self) -> &u32 {
            match &self.max {
                Max::NegativeInfinity => &u32::min_value(),
                Max::Maximum(m) => m,
            }
        }
    }

    impl<S> Annotation<NStack<u32, MaxAndCardinality, S>, S> for MaxAndCardinality
    where
        S: Store,
    {
        fn identity() -> Self {
            let cardinality = <Cardinality as Annotation<
                NStack<u32, MaxAndCardinality, S>,
                S,
            >>::identity();
            let max = <Max<u32> as Annotation<
                NStack<u32, MaxAndCardinality, S>,
                S,
            >>::identity();

            Self { cardinality, max }
        }

        fn from_leaf(leaf: &u32) -> Self {
            let cardinality = <Cardinality as Annotation<
                NStack<u32, MaxAndCardinality, S>,
                S,
            >>::from_leaf(leaf);

            let max = <Max<u32> as Annotation<
                NStack<u32, MaxAndCardinality, S>,
                S,
            >>::from_leaf(leaf);

            Self { cardinality, max }
        }

        fn from_node(node: &NStack<u32, MaxAndCardinality, S>) -> Self {
            let cardinality = <Cardinality as Annotation<
                NStack<u32, MaxAndCardinality, S>,
                S,
            >>::from_node(node);

            let max = <Max<u32> as Annotation<
                NStack<u32, MaxAndCardinality, S>,
                S,
            >>::from_node(node);

            Self { cardinality, max }
        }
    }

    #[test]
    fn test_max_annotation() -> Result<(), <MemStore as Store>::Error> {
        let mut stack: NStack<u32, MaxAndCardinality, MemStore> = NStack::new();

        let n = 5;

        for i in 0..n {
            stack.push(i)?;

            println!("{}", i);
        }

        Ok(())
    }
}
