//! Implementations of iterator-related traits
use std::{
    borrow::Borrow,
    hash::{BuildHasher, Hash},
};

use dashmap::{mapref::multiple::RefMulti, DashMap};

use crate::Graph;

#[cfg(feature = "rayon")]
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator};

#[cfg(feature = "rayon")]
impl<'a, K, V, E, S> Graph<K, V, E, S>
where
    K: 'a + Eq + Hash + Send + Sync,
    V: 'a + Send + Sync,
    E: 'a + Send + Sync,
    S: BuildHasher + Clone + Send + Sync,
{
    pub fn par_iter_nodes(
        &'a self,
    ) -> impl rayon::iter::IntoParallelIterator<Item = RefMulti<'a, K, V>> + 'a {
        self.nodes.par_iter()
    }
}

impl<K, V, E, S> FromIterator<(K, V)> for Graph<K, V, E, S>
where
    K: Eq + Hash,
    S: Default + BuildHasher + Clone,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let hint = iter.size_hint();
        let capacity = hint.1.unwrap_or(hint.0);
        let graph = Graph::with_capacity_and_hasher(capacity, S::default());
        for (key, value) in iter {
            graph.insert(key, value);
        }
        graph
    }
}

impl<K, V, E, S> FromIterator<V> for Graph<K, V, E, S>
where
    K: Eq + Hash + Clone,
    V: Borrow<K>,
    S: Default + BuildHasher + Clone,
{
    fn from_iter<T: IntoIterator<Item = V>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let hint = iter.size_hint();
        let capacity = hint.1.unwrap_or(hint.0);
        let graph: Graph<K, V, E, S> = Graph::with_capacity_and_hasher(capacity, S::default());
        for value in iter {
            let key: K = value.borrow().clone();
            graph.insert(key, value);
        }
        graph
    }
}

impl<'a, K, V, E, S> IntoIterator for &'a Graph<K, V, E, S>
where
    K: 'a + Eq + Hash,
    V: 'a,
    E: 'a,
    S: BuildHasher + Clone,
{
    type Item = RefMulti<'a, K, V>;
    type IntoIter = <&'a DashMap<K, V, S> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter()
    }
}

#[cfg(feature = "rayon")]
impl<'a, K, V, E, S> IntoParallelIterator for &'a Graph<K, V, E, S>
where
    K: 'a + Eq + Hash + Send + Sync,
    V: 'a + Send + Sync,
    E: 'a + Send + Sync,
    S: BuildHasher + Clone + Send + Sync,
{
    type Item = RefMulti<'a, K, V>;
    type Iter = <&'a DashMap<K, V, S> as IntoParallelIterator>::Iter;
    fn into_par_iter(self) -> Self::Iter {
        IntoParallelIterator::into_par_iter(&self.nodes)
    }
}
