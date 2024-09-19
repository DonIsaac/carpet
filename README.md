# Weave

Weave is a thread-safe, fully-parallelized directed graph designed for
write-heavy multi-threaded jobs.

Graphs directed, and both nodes and edges may store arbitrary data.

```rust
use weave::Graph;

type UserId = u64;
struct User {
    id: UserId,
    name: String,
}

pub enum Relationship {
    Follows,
    Blocks,
}

impl std::borrow::Borrow<UserId> for User {
    fn borrow(&self) -> &UserId {
        &self.id
    }
}

let users: Graph<UserId, User, Relationship> = [
    User { id: 1, name: "Alice".to_string()     },
    User { id: 2, name: "Bob".to_string()       },
    User { id: 3, name: "Charlie".to_string()   },
].into_iter().collect();

// Alice follows Bob, and Bob blocks Charlie
users.add_edge(1, 2, Relationship::Follows);
users.add_edge(2, 3, Relationship::Blocks);
```

## Installation

Weave is available on [crates.io](https://crates.io/weave)

```sh
cargo add weave
```

## Thread-Safety

Weave graphs can be safely mutated across threads. Most methods have a
corresponding `*_mut` method that takes a read-only `&self` reference but allows
for writes to the graph.

Weave uses [dashmap](https://crates.io/dashmap) for storing graph data, which
features shared-based locking allowing for concurrent writes to different
sections of the graph. However, this introduces a footgun: attempts to obtain
two write references, or one read and one write reference, to the same graph
data (e.g. nodes, edges) on the same thread will result in a deadlock. A single
reference on different threads is safe. Refer to dashmap's [deadlocking
documentation](https://docs.rs/dashmap) for more information.

## Parallelism

Weave is intended to be used with [rayon](https://crates.io/rayon), and
implements rayon's [parallel iterator
traits](https://docs.rs/rayon/latest/rayon/iter/index.html).

```rust
use weave::Graph;
use rayon::prelude::*;

let graph: Graph<i32, i32> = (0..100).map(|i| (i, i)).collect();
graph.par_iter().for_each(|entry| {
    assert_eq!(entry.key(), entry.value());
})
```

Weave does not force you to use rayon if you prefer a different parallelism
crate. All rayon-related methods and trait implementations can be disabled by
turning off the `rayon` feature.

```toml
[dependencies]
weave = { version = "*", default-features = false }
```

## Debugging

Graphs can be printed to GraphViz's [dot](https://graphviz.org/) format for
debugging by enabling the `dot` feature.

```toml
[dependencies]
weave = { version = "*", features = ["dot"] }
```

```rust
use std::{fs, io::{self, Write}};
use weave::{dot::ToDot, Graph};

// Save the dot graph to a file, then run `dot -Tpng debug.dot -o debug.png`
fn save_to_file() -> io::Result<()> {
    let mut debug_file = fs::File::create("debug.dot")?;
    let graph = Graph::<i32, i32>::default();
    graph.to_dot(&mut debug_file)?;
    debug_file.flush()
}

fn save_to_string() -> io::Result<String> {
    let mut buf: Vec<u8> = Vec::new();
    let graph = Graph::<i32, i32>::default();
    graph.to_dot(&mut buf)?;
    Ok(String::from_utf8(buf).unwrap())
}

let _ = save_to_string().unwrap();
```

Implement `std::fmt::Display` on your graph's data types to customize the
output.  

## Performance

Weave trades conccurrent write performance for memory efficiency. To combat some
of this, Weave provides several optimization methods for use cases that have a
write-heavy phase and a read-heavy phase.

### Freeing Memory

After your graph has been fully constructed, you can
use `shrink_to_fit` or `shrink_all_to_fit` to free excess memory. The latter is
a more aggressive version that frees memory in all edge lists, resulting in
better compression at the cost of performance.

```rust
use weave::Graph;
let mut graph: Graph<i32, i32> = (0..100).map(|i| (i, i)).collect();

graph.shrink_to_fit();
graph.shrink_all_to_fit();
```

Note that `shrink_to_fit` and `shrink_all_to_fit` are the only methods requiring
a `&mut self` reference. This prevents your multi-threaded jobs from performing
costly frees. Delegate these calls to the main thread after all mutations have
completed.

### Read-Only Mode
When you are done mutating your graph, you can create a `ReadOnlyGraph`. This
enables:
1. A friendlier raw-reference based API instead of dashmap's `Ref` and `RefMut`
2. Trait implementations that are otherwise impossible to add, such as `Index`

## License
Weave is available under the MIT license. You may find a copy in
[LICENSE](./LICENSE)
