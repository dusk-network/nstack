![Dusk CI](https://github.com/dusk-network/nstack/actions/workflows/dusk_ci.yml/badge.svg)
[![codecov](https://codecov.io/gh/dusk-network/nstack/branch/master/graph/badge.svg?token=GQUFNVJXT1)](https://codecov.io/gh/dusk-network/nstack)

# nstack

nstack is a stack-like merkle datastructure for storing and accessing indexed values.

Operations supported are add and remove at the end of the structure, and mutable access to indexed leaves.

## Usage example
```rust
use nstack::annotation::Cardinality;
use nstack::NStack;

let mut nt = NStack::<i32, Cardinality>::new();

nt.push(0);
nt.push(1);

// mutable references to the last element
if let Some(mut branch) = nt.nth_mut(1) {
    *branch = 2;
}

assert_eq!(nt.pop(), Some(2));
assert_eq!(nt.pop(), Some(0));
```

