# nstack

nstack is a stack-like merkle datastructure for storing and accessing indexed values.

Operations supported are add and remove at the end of the structure, and mutable access to indexed leaves.

# Usage example

```rust

use kelvin::{annotations::Cardinality, Blake2b};

let n: usize = 256;
let mut nt = NStack::<_, Cardinality<u64>, Blake2b>::new();

for i in 0..n {
    nt.push(i).unwrap();
}

// get a mutable reference to the 128'th element

let element = &mut *nt.get_mut(128).unwrap().unwrap();

```

