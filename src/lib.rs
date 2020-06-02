//! NStack
//!
//! A stack datastructure with indexed lookup.
#![warn(missing_docs)]
use std::borrow::Borrow;
use std::marker::PhantomData;
use std::{io, mem};

use kelvin::annotations::{Cardinality, Counter, Nth};
use kelvin::{
    Annotation, Branch, BranchMut, ByteHash, Compound, Content, Handle,
    HandleMut, HandleType, Sink, Source,
};

const N: usize = 4;

/// A stack datastructure with indexed lookup.
#[derive(Clone)]
pub struct NStack<T, A, H>([Handle<Self, H>; N], PhantomData<(T, A)>)
where
    T: Content<H>,
    Self: Compound<H>,
    H: ByteHash;

impl<T, A, H> Default for NStack<T, A, H>
where
    T: Content<H>,
    A: Content<H> + Annotation<T, H>,
    H: ByteHash,
{
    fn default() -> Self {
        let handles: [Handle<Self, H>; N] = Default::default();
        NStack(handles, PhantomData)
    }
}

impl<T, A, H> Content<H> for NStack<T, A, H>
where
    T: Content<H>,
    A: Content<H> + Annotation<T, H>,
    H: ByteHash,
{
    fn persist(&mut self, sink: &mut Sink<H>) -> io::Result<()> {
        for handle in self.0.iter_mut() {
            handle.persist(sink)?;
        }
        Ok(())
    }

    fn restore(source: &mut Source<H>) -> io::Result<Self> {
        let mut handles: [Handle<Self, H>; N] = Default::default();
        for handle in handles.iter_mut() {
            *handle = Handle::restore(source)?;
        }
        Ok(NStack(handles, PhantomData))
    }
}

impl<T, A, H> Compound<H> for NStack<T, A, H>
where
    T: Content<H>,
    A: Content<H> + Annotation<T, H>,
    H: ByteHash,
{
    type Leaf = T;
    type Annotation = A;

    fn children(&self) -> &[Handle<Self, H>] {
        &self.0
    }

    fn children_mut(&mut self) -> &mut [Handle<Self, H>] {
        &mut self.0
    }
}

enum PushResult<T> {
    Ok,
    NoRoom(T, usize),
}

enum PopResult<T> {
    Ok(T),
    Last(T),
    None,
}

impl<T, A, H> NStack<T, A, H>
where
    T: Content<H>,
    A: Content<H> + Annotation<T, H>,
    H: ByteHash,
{
    /// Creates a new empty NStack
    pub fn new() -> Self {
        Self::default()
    }

    /// Pushes a new element onto the stack
    pub fn push(&mut self, t: T) -> io::Result<()> {
        match self._push(t)? {
            PushResult::Ok => Ok(()),
            PushResult::NoRoom(t, _) => {
                // in this branch we determined that the node is full with leaves or nodes,
                // so we just wrap it in a new root node and recurse

                let old_root = mem::take(self);

                // the first child of our new root will be our old root
                self.0[0] = Handle::new_node(old_root);
                // recurse
                self.push(t)
            }
        }
    }

    fn _push(&mut self, t: T) -> io::Result<PushResult<T>> {
        #[derive(Debug)]
        enum State {
            Initial,
            SeenNode(usize),
        }
        use State::*;

        let mut state = Initial;

        for i in 0..N {
            match (&state, self.0[i].handle_type()) {
                (Initial, HandleType::None) => {
                    self.0[i] = Handle::new_leaf(t);
                    return Ok(PushResult::Ok);
                }
                (Initial, HandleType::Leaf) => (),
                (Initial, HandleType::Node) => state = SeenNode(i),
                (SeenNode(_), HandleType::None) => {
                    // we found the last node
                    break;
                }
                (SeenNode(_), HandleType::Leaf) => {
                    unreachable!("invariant: no nodes and leaves on same level")
                }
                (SeenNode(_), HandleType::Node) => state = SeenNode(i),
            }
        }

        match state {
            Initial => Ok(PushResult::NoRoom(t, 0)),
            SeenNode(i) => {
                let insert_new;

                match &mut *self.0[i].inner_mut()? {
                    HandleMut::Node(n) => {
                        match n._push(t)? {
                            PushResult::Ok => return Ok(PushResult::Ok),
                            PushResult::NoRoom(t, depth) => {
                                // we need to create a new branch
                                // is there space here?
                                if i == N - 1 {
                                    // no space for new branch
                                    return Ok(PushResult::NoRoom(
                                        t,
                                        depth + 1,
                                    ));
                                } else {
                                    let mut new_node = Self::new();
                                    new_node.0[0] = Handle::new_leaf(t);

                                    // wrap the node in a long enough branch

                                    for _ in 0..depth {
                                        let inner = mem::replace(
                                            &mut new_node,
                                            Self::new(),
                                        );
                                        new_node.0[0] = Handle::new_node(inner);
                                    }

                                    // TODO extend branch
                                    insert_new = Some(new_node);
                                }
                            }
                        }
                    }
                    _ => unreachable!("Seen node"),
                }

                if let Some(new_node) = insert_new {
                    self.0[i + 1] = Handle::new_node(new_node);
                    Ok(PushResult::Ok)
                } else {
                    unreachable!();
                }
            }
        }
    }

    /// Pop an element off the stack.
    ///
    /// Returns the popped element, if any.
    pub fn pop(&mut self) -> io::Result<Option<T>> {
        match self._pop()? {
            PopResult::Ok(t) | PopResult::Last(t) => Ok(Some(t)),
            PopResult::None => Ok(None),
        }
    }

    fn _pop(&mut self) -> io::Result<PopResult<T>> {
        for i in 0..N {
            // reverse iteration
            let i = N - i - 1;

            match self.0[i].handle_type() {
                HandleType::None => (),
                HandleType::Leaf => {
                    let popped =
                        mem::replace(&mut self.0[i], Handle::new_empty())
                            .into_leaf();

                    // did we remove the last element?
                    return Ok(if i == 0 {
                        PopResult::Last(popped)
                    } else {
                        PopResult::Ok(popped)
                    });
                }
                HandleType::Node => {
                    let mut remove = false;
                    let popped;

                    match &mut *self.0[i].inner_mut()? {
                        HandleMut::Node(n) => {
                            match n._pop()? {
                                PopResult::Ok(t) => popped = Some(t),
                                PopResult::Last(t) => {
                                    popped = Some(t);
                                    remove = true;
                                }
                                PopResult::None => {
                                    unreachable!("invariant: no empty subnodes")
                                }
                            };
                        }
                        _ => unreachable!(
                            "invariant: no nodes and leaves on same level"
                        ),
                    }

                    return Ok(match (remove, popped) {
                        (_, None) => PopResult::None,
                        (false, Some(t)) => PopResult::Ok(t),
                        (true, Some(t)) => {
                            self.0[i] = Handle::new_empty();
                            if i == 0 {
                                PopResult::Last(t)
                            } else {
                                PopResult::Ok(t)
                            }
                        }
                    });
                }
            }
        }
        Ok(PopResult::None)
    }

    /// Get a branch pointing to the element stored at index `idx`, if any
    pub fn get<U>(&self, idx: U) -> io::Result<Option<Branch<Self, H>>>
    where
        U: Counter,
        <Self as Compound<H>>::Annotation: Borrow<Cardinality<U>>,
    {
        Branch::new(self, &mut Nth::new(idx))
    }

    /// Get a mutable branch pointing to the element stored at index `idx`, if any
    pub fn get_mut<U>(
        &mut self,
        idx: U,
    ) -> io::Result<Option<BranchMut<Self, H>>>
    where
        U: Counter,
        <Self as Compound<H>>::Annotation: Borrow<Cardinality<U>>,
    {
        BranchMut::new(self, &mut Nth::new(idx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use kelvin::{quickcheck_stack, Blake2b};

    #[test]
    fn trivial() {
        let mut nt = NStack::<_, Cardinality<u64>, Blake2b>::new();
        nt.push(8).unwrap();
        assert_eq!(nt.pop().unwrap(), Some(8));
    }

    #[test]
    fn double() {
        let mut nt = NStack::<_, Cardinality<u64>, Blake2b>::new();
        nt.push(0).unwrap();
        nt.push(1).unwrap();
        assert_eq!(nt.pop().unwrap(), Some(1));
        assert_eq!(nt.pop().unwrap(), Some(0));
    }

    #[test]
    fn multiple() {
        let n = 1024;

        let mut nt = NStack::<_, Cardinality<u64>, Blake2b>::new();

        for i in 0..n {
            nt.push(i).unwrap();
        }

        for i in 0..n {
            assert_eq!(nt.pop().unwrap(), Some(n - i - 1));
        }

        assert_eq!(nt.pop().unwrap(), None);
        nt.assert_correct_empty_state();
    }

    #[test]
    fn get() {
        let n = 128;

        let mut nt = NStack::<_, Cardinality<u64>, Blake2b>::new();

        for i in 0..n {
            println!("pushing {}", i);
            nt.push(i).unwrap();

            for o in 0..i {
                assert_eq!(*nt.get(o).unwrap().unwrap(), o);
            }
            assert!(nt.get(i + 1).unwrap().is_none());
        }
    }

    #[test]
    fn get_mut() {
        let n = 1024;

        let mut nt = NStack::<_, Cardinality<u64>, Blake2b>::new();

        for i in 0..n {
            nt.push(i).unwrap();
        }

        for i in 0..n {
            *nt.get_mut(i).unwrap().unwrap() += 1;
        }

        for i in 0..n {
            assert_eq!(*nt.get(i).unwrap().unwrap(), i + 1);
        }
    }

    // Assert that all branches are always of the same length
    #[test]
    fn branch_lengths() {
        let n = 256;

        let mut nt = NStack::<_, Cardinality<u64>, Blake2b>::new();

        for i in 0..n {
            nt.push(i).unwrap();
        }

        let length_zero = nt.get(0).unwrap().unwrap().levels().len();

        for i in 1..n {
            assert_eq!(length_zero, nt.get(i).unwrap().unwrap().levels().len())
        }
    }

    quickcheck_stack!(|| NStack::<_, Cardinality<u64>, Blake2b>::new());
}
