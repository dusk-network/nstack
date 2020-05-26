
# nstack

nstack is a stack merkle datastructure for storing and accessing indexed values.

It builds on [kelvin](https://github.com/dusk-network/kelvin) for database backing and merkle operations.

# Usage example

```rust

use kelvin::{annotations::Cardinality, Blake2b};

let n: usize = 256;
let mut nt = NTree::<_, Cardinality<u64>, Blake2b>::new();

for i in 0..n {
    nt.push(i).unwrap();
}

// get a mutable reference to the 128'th element

let element = &mut *nt.get_mut(128).unwrap().unwrap();

```

# Structure

The general structure of the $TREE_NAME is a 4-way splitting tree that is always populated from the "bottom left".

As new nodes are added, they get added to the right of the last leaf, creating a new root level on top when neccesary.

The leaves of the tree are always located at the same depth relative to the root.

```
containing 1 element:

[0, - ]

containing 3 elements:

  [ *,  * ]
   /     \
[0, 1] [2, -]

containing 5 elements:

      [ *    ,    *  ]
       /           \
  [ * , * ]      [ * , - ]
   /     \        /
[0, 1] [2, 3] [ 4, - ]
```