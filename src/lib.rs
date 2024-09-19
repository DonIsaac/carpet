#![doc = include_str!("../README.md")]
mod iter;
mod read_only;

#[cfg(test)]
mod test;

#[cfg(feature = "dot")]
pub mod dot;

extern crate dashmap;
extern crate nohash_hasher;

use std::{
    borrow::Borrow,
    collections::hash_map::RandomState,
    fmt::{self, Debug},
    hash::{BuildHasher, Hash, Hasher},
    sync::atomic::AtomicU64,
};

use dashmap::{
    mapref::{
        multiple::RefMulti,
        one::{Ref, RefMut},
    },
    DashMap,
};
use nohash_hasher::{BuildNoHashHasher, IsEnabled};
use read_only::ReadOnlyGraph;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct EdgeId(u64);
impl Hash for EdgeId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.0);
    }
}

impl IsEnabled for EdgeId {}

type EdgeList<K, S> = DashMap<K, Vec<(EdgeId, K)>, S>;
type EdgeHasher = BuildNoHashHasher<EdgeId>;
type DefaultHasher = RandomState;

pub struct Graph<K, V, E = (), S = DefaultHasher> {
    nodes: DashMap<K, V, S>,
    edges: DashMap<EdgeId, E, EdgeHasher>,
    to: EdgeList<K, S>,
    from: EdgeList<K, S>,
    curr_edge_id: AtomicU64,
}

impl<K, V, E, S> Default for Graph<K, V, E, S>
where
    K: Eq + Hash,
    S: Default + BuildHasher + Clone,
{
    fn default() -> Self {
        Self {
            nodes: DashMap::default(),
            edges: DashMap::with_hasher(EdgeHasher::default()),
            to: DashMap::default(),
            from: DashMap::default(),
            curr_edge_id: AtomicU64::new(0),
        }
    }
}
impl<'a, K: 'a + Eq + Hash, V: 'a, E: 'a> Graph<K, V, E, DefaultHasher> {
    pub fn new() -> Self {
        Self::with_hasher(DefaultHasher::default())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, DefaultHasher::default())
    }
}

impl<'a, K, V, E, S> Graph<K, V, E, S>
where
    K: 'a + Eq + Hash,
    V: 'a,
    E: 'a,
    S: BuildHasher + Clone,
{
    pub fn with_hasher(hasher: S) -> Self {
        Self::with_capacity_and_hasher(0, hasher)
    }

    pub fn with_capacity_and_hasher(capacity: usize, hasher: S) -> Self {
        // Assuming a fully-connected graph with even to/from distribution.
        // TODO: validate this assumption
        let edge_capacity = capacity / 2;
        Self {
            nodes: DashMap::with_capacity_and_hasher(capacity, hasher.clone()),
            edges: DashMap::with_capacity_and_hasher(capacity, EdgeHasher::default()),
            to: DashMap::with_capacity_and_hasher(edge_capacity, hasher.clone()),
            from: DashMap::with_capacity_and_hasher(edge_capacity, hasher),
            curr_edge_id: AtomicU64::new(0),
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn has_node<Q>(&'a self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.nodes.contains_key(key)
    }

    pub fn get_node<Q>(&'a self, key: &Q) -> Option<Ref<'a, K, V>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.nodes.get(key)
    }

    pub fn get_node_mut<Q>(&'a self, key: &Q) -> Option<RefMut<'a, K, V>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.nodes.get_mut(key)
    }

    /// Inserts a node into the graph under the given `key`. Returns the old value associated with the key if there was one.
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        self.nodes.insert(key, value)
    }

    /// Add an edge between two existing nodes, originating at `from` and terminating at `to`.
    pub fn add_edge(&self, from: K, to: K, edge: E)
    where
        K: Clone,
        // V: Default,
        S: Default,
    {
        debug_assert!(self.nodes.contains_key(&from));
        debug_assert!(self.nodes.contains_key(&to));
        let edge_id = self.next_edge_id();
        self.edges.insert(edge_id, edge);

        self.from
            .entry(from.clone())
            .or_default()
            .push((edge_id, to.clone()));
        self.to.entry(to).or_default().push((edge_id, from));
    }

    pub fn iter_nodes(&'a self) -> impl Iterator<Item = RefMulti<'a, K, V>> + 'a {
        self.nodes.iter()
    }

    pub fn shrink_to_fit(&mut self) {
        self.nodes.shrink_to_fit();
        self.edges.shrink_to_fit();
        self.to.shrink_to_fit();
        self.from.shrink_to_fit();
    }

    pub fn shrink_all_to_fit(&mut self) {
        self.nodes.shrink_to_fit();
        self.edges.shrink_to_fit();
        for mut to in self.to.iter_mut() {
            to.shrink_to_fit();
        }
        self.to.shrink_to_fit();

        for mut from in self.from.iter_mut() {
            from.shrink_to_fit();
        }
        self.from.shrink_to_fit();
    }

    pub fn into_read_only(self) -> ReadOnlyGraph<K, V, E, S> {
        ReadOnlyGraph {
            nodes: self.nodes.into_read_only(),
            edges: self.edges.into_read_only(),
            to: self.to.into_read_only(),
            from: self.from.into_read_only(),
        }
    }
}

impl<K, V, E, S> Clone for Graph<K, V, E, S>
where
    K: Eq + Hash + Clone,
    V: Clone,
    E: Clone,
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
            to: self.to.clone(),
            from: self.from.clone(),
            curr_edge_id: AtomicU64::new(
                self.curr_edge_id.load(std::sync::atomic::Ordering::Relaxed),
            ),
        }
    }
}

impl<K, V, E, S> Graph<K, V, E, S> {
    pub(self) fn next_edge_id(&self) -> EdgeId {
        EdgeId(
            self.curr_edge_id
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        )
    }
}

#[cfg(feature = "dot")]
impl<K, V, E, S> dot::ToDot for Graph<K, V, E, S>
where
    K: fmt::Display + Eq + Hash,
    V: fmt::Display + Sized,
    // E: fmt::Display,
    S: BuildHasher + Clone,
{
    fn to_dot<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        const INDENT: &str = "  ";
        // let mut dot: DotBuilder<K>
        writeln!(writer, "digraph G {{")?;
        writeln!(writer, "{INDENT}rankdir=LR;")?;
        writer.write_all(b"\n")?;

        for node in &self.nodes {
            writeln!(
                writer,
                "{}{} [label=\"{}\"];",
                INDENT,
                node.key(),
                node.value()
            )?;
        }

        writer.write_all(b"\n")?;

        for froms_ref in &self.from {
            let froms = froms_ref.value();
            let from = froms_ref.key();
            for (_, to) in froms.iter() {
                writeln!(writer, "{}{} -> {};", INDENT, from, to)?;
            }
        }
        writer.write_all(b"\n}")?;
        writer.flush()
    }
}

impl<K, V, E, S> Debug for Graph<K, V, E, S>
where
    K: Debug + Eq + Hash,
    V: Debug,
    E: Debug,
    S: BuildHasher + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DiGraphEdged")
            .field("nodes", &self.nodes)
            .field("edges", &self.edges)
            .field("to", &self.to)
            .field("from", &self.from)
            .finish()
    }
}

#[test]
fn test_from_iter() {
    use crate::Graph;

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
        User {
            id: 1,
            name: "Alice".to_string(),
        },
        User {
            id: 2,
            name: "Bob".to_string(),
        },
        User {
            id: 3,
            name: "Charlie".to_string(),
        },
    ]
    .into_iter()
    .collect();

    // Alice follows Bob, and Bob blocks Charlie
    users.add_edge(1, 2, Relationship::Follows);
    users.add_edge(2, 3, Relationship::Blocks);
}
