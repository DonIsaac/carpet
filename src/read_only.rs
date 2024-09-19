use std::{
    borrow::Borrow,
    hash::{BuildHasher, Hash},
    ops,
};

use dashmap::ReadOnlyView;

use crate::{EdgeHasher, EdgeId};

pub struct ReadOnlyGraph<K, V, E, S> {
    pub(crate) nodes: ReadOnlyView<K, V, S>,
    pub(crate) edges: ReadOnlyView<EdgeId, E, EdgeHasher>,
    pub(crate) to: ReadOnlyView<K, Vec<(EdgeId, K)>, S>,
    pub(crate) from: ReadOnlyView<K, Vec<(EdgeId, K)>, S>,
    // note: curr_edge_id not needed since no more edges will be added
}

impl<'a, K, V, E, S> ReadOnlyGraph<K, V, E, S>
where
    K: 'a + Eq + Hash,
    V: 'a,
    E: 'a,
    S: 'a + BuildHasher + Clone,
{
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn has_node<Q>(&'a self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.nodes.contains_key(key)
    }

    pub fn get_node<Q>(&'a self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.nodes.get(key)
    }

    pub fn get_edge(&self, key: EdgeId) -> Option<&E> {
        self.edges.get(&key)
    }

    pub fn edge_ids_from<Q>(&'a self, key: &Q) -> Option<&Vec<(EdgeId, K)>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.from.get(key)
    }

    pub fn edge_ids_to<Q>(&'a self, key: &Q) -> Option<&Vec<(EdgeId, K)>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.to.get(key)
    }

    pub fn iter_edges_from<Q>(&'a self, key: &Q) -> Option<impl Iterator<Item = (&V, &E, &V)> + 'a>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let from = self.nodes.get(key)?;
        let edges_from = self.edge_ids_from(key)?;
        Some(edges_from.iter().map(move |(edge_id, to)| {
            (
                from,
                self.edges.get(edge_id).unwrap(),
                self.nodes.get(to.borrow()).unwrap(),
            )
        }))
    }

    pub fn iter_nodes(&self) -> impl Iterator<Item = (&K, &V)> {
        self.nodes.iter()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&V, &E, &V)> {
        self.nodes
            .iter()
            .filter_map(move |(key, value)| self.from.get(key).map(|edges| (value, edges)))
            .flat_map(move |(from, edges)| {
                edges.iter().map(move |(edge_id, to_id)| {
                    let edge = self.edges.get(edge_id).unwrap();
                    let to = self.nodes.get(to_id).unwrap();
                    (from, edge, to)
                })
            })
    }
}

impl<'a, K, V, E, S> ops::Index<K> for ReadOnlyGraph<K, V, E, S>
where
    K: 'a + Eq + Hash,
    V: 'a,
    E: 'a,
    S: 'a + BuildHasher + Clone,
{
    type Output = V;
    fn index(&self, key: K) -> &V {
        self.get_node(&key).expect("Key not found")
    }
}
