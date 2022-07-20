![Dusk CI](https://github.com/dusk-network/nstack/actions/workflows/dusk_ci.yml/badge.svg)
[![codecov](https://codecov.io/gh/dusk-network/nstack/branch/master/graph/badge.svg?token=GQUFNVJXT1)](https://codecov.io/gh/dusk-network/nstack)

# nstack

nstack is a stack-like merkle datastructure for storing and accessing indexed values.

Operations supported are add and remove at the end of the structure, and mutable access to indexed leaves.

## Usage example
```rust
use nstack::NStack;

let mut nt = NStack::<i32, ()>::new();

nt.push(0);
nt.push(1);

assert_eq!(nt.pop(), Some(1));
assert_eq!(nt.pop(), Some(0));
```

