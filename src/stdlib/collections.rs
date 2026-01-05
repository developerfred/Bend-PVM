#![allow(clippy::ptr_arg)]
#![allow(clippy::new_ret_no_self)]

/// Collection utilities for Bend-PVM
pub struct Collections;

/// Vector operations
pub struct VecUtils;

impl VecUtils {
    /// Create new vector
    pub fn new<T>() -> Vec<T> {
        Vec::new()
    }

    /// Create vector with capacity
    pub fn with_capacity<T>(capacity: usize) -> Vec<T> {
        Vec::with_capacity(capacity)
    }

    /// Get length
    pub fn len<T>(v: &Vec<T>) -> usize {
        v.len()
    }

    /// Check if empty
    pub fn is_empty<T>(v: &Vec<T>) -> bool {
        v.is_empty()
    }

    /// Get element at index
    pub fn get<T>(v: &Vec<T>, index: usize) -> Option<&T> {
        v.get(index)
    }

    /// Push element
    pub fn push<T>(v: &mut Vec<T>, item: T) {
        v.push(item);
    }

    /// Pop element
    pub fn pop<T>(v: &mut Vec<T>) -> Option<T> {
        v.pop()
    }

    /// Get first element
    pub fn first<T>(v: &Vec<T>) -> Option<&T> {
        v.first()
    }

    /// Get last element
    pub fn last<T>(v: &Vec<T>) -> Option<&T> {
        v.last()
    }

    /// Insert at index
    pub fn insert<T>(v: &mut Vec<T>, index: usize, item: T) {
        v.insert(index, item);
    }

    /// Remove at index
    pub fn remove<T>(v: &mut Vec<T>, index: usize) -> T {
        v.remove(index)
    }

    /// Clear vector
    pub fn clear<T>(v: &mut Vec<T>) {
        v.clear();
    }

    /// Contains element
    pub fn contains<T>(v: &Vec<T>, item: &T) -> bool
    where
        T: PartialEq,
    {
        v.contains(item)
    }

    /// Find index of element
    pub fn find_index<T>(v: &Vec<T>, item: &T) -> Option<usize>
    where
        T: PartialEq,
    {
        v.iter().position(|x| x == item)
    }

    /// Filter elements
    pub fn filter<T>(v: &Vec<T>, predicate: impl Fn(&T) -> bool) -> Vec<T>
    where
        T: Clone,
    {
        v.iter().filter(|x| predicate(*x)).cloned().collect()
    }

    /// Map elements
    pub fn map<T, U>(v: &Vec<T>, mapper: impl Fn(&T) -> U) -> Vec<U> {
        v.iter().map(mapper).collect()
    }

    /// Reduce elements
    pub fn reduce<T>(v: &Vec<T>, reducer: impl Fn(Option<&T>, &T) -> T) -> Option<T>
    where
        T: Clone,
    {
        let mut iter = v.iter();
        let first = iter.next()?;
        let mut result = first.clone();
        for item in iter {
            result = reducer(Some(&result), item);
        }
        Some(result)
    }

    /// Sort vector
    pub fn sort<T>(v: &mut Vec<T>)
    where
        T: Ord,
    {
        v.sort();
    }

    /// Sort by key
    pub fn sort_by<T, K>(v: &mut Vec<T>, key: impl Fn(&T) -> K)
    where
        K: Ord,
    {
        v.sort_by_key(key);
    }

    /// Reverse vector
    pub fn reverse<T>(v: &mut Vec<T>) {
        v.reverse();
    }

    /// Get all elements as a slice
    pub fn to_slice<T>(v: &Vec<T>) -> &[T] {
        v.as_slice()
    }

    /// Get all elements as a mutable slice
    pub fn to_mut_slice<T>(v: &mut Vec<T>) -> &mut [T] {
        v.as_mut_slice()
    }

    /// Extend from slice
    pub fn extend_from_slice<T>(v: &mut Vec<T>, slice: &[T])
    where
        T: Clone,
    {
        v.extend_from_slice(slice);
    }

    /// Split off at index
    pub fn split_off<T>(v: &mut Vec<T>, at: usize) -> Vec<T>
    where
        T: Clone,
    {
        v.split_off(at)
    }

    /// Append another vector
    pub fn append<T>(v: &mut Vec<T>, other: &mut Vec<T>) {
        v.append(other);
    }

    /// Drain a range
    pub fn drain<T>(
        v: &mut Vec<T>,
        range: impl std::ops::RangeBounds<usize>,
    ) -> std::vec::Drain<'_, T> {
        v.drain(range)
    }

    /// Get values as Vec
    pub fn values<K, V>(map: &std::collections::HashMap<K, V>) -> Vec<V>
    where
        V: Clone,
    {
        map.values().cloned().collect()
    }

    /// Get keys as Vec
    pub fn keys<K, V>(map: &std::collections::HashMap<K, V>) -> Vec<K>
    where
        K: Clone,
    {
        map.keys().cloned().collect()
    }

    /// Get entries as Vec of tuples
    pub fn entries<K, V>(map: &std::collections::HashMap<K, V>) -> Vec<(K, V)>
    where
        K: Clone,
        V: Clone,
    {
        map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    /// Insert all from vector
    pub fn insert_all<K, V>(map: &mut std::collections::HashMap<K, V>, entries: &[(K, V)])
    where
        K: Clone + std::hash::Hash + Eq,
        V: Clone,
    {
        for (k, v) in entries {
            map.insert(k.clone(), v.clone());
        }
    }
}

/// Map operations
pub struct MapUtils;

impl MapUtils {
    /// Create new HashMap
    pub fn new<K, V>() -> std::collections::HashMap<K, V> {
        std::collections::HashMap::new()
    }

    /// Create with capacity
    pub fn with_capacity<K, V>(capacity: usize) -> std::collections::HashMap<K, V> {
        std::collections::HashMap::with_capacity(capacity)
    }

    /// Insert key-value
    pub fn insert<K, V>(map: &mut std::collections::HashMap<K, V>, key: K, value: V) -> Option<V>
    where
        K: std::hash::Hash + Eq,
    {
        map.insert(key, value)
    }

    /// Get value by key
    pub fn get<'a, K, V>(map: &'a std::collections::HashMap<K, V>, key: &K) -> Option<&'a V>
    where
        K: std::hash::Hash + Eq,
    {
        map.get(key)
    }

    /// Check if key exists
    pub fn contains<K, V>(map: &std::collections::HashMap<K, V>, key: &K) -> bool
    where
        K: std::hash::Hash + Eq,
    {
        map.contains_key(key)
    }

    /// Remove key
    pub fn remove<K, V>(map: &mut std::collections::HashMap<K, V>, key: &K) -> Option<V>
    where
        K: std::hash::Hash + Eq,
    {
        map.remove(key)
    }

    /// Get length
    pub fn len<K, V>(map: &std::collections::HashMap<K, V>) -> usize {
        map.len()
    }

    /// Check if empty
    pub fn is_empty<K, V>(map: &std::collections::HashMap<K, V>) -> bool {
        map.is_empty()
    }

    /// Clear map
    pub fn clear<K, V>(map: &mut std::collections::HashMap<K, V>) {
        map.clear();
    }

    /// Get all keys
    pub fn keys<K, V>(map: &std::collections::HashMap<K, V>) -> Vec<K>
    where
        K: Clone,
    {
        map.keys().cloned().collect()
    }

    /// Get all values
    pub fn values<K, V>(map: &std::collections::HashMap<K, V>) -> Vec<V>
    where
        V: Clone,
    {
        map.values().cloned().collect()
    }

    /// Iterate over entries
    pub fn entries<K, V>(map: &std::collections::HashMap<K, V>) -> Vec<(K, V)>
    where
        K: Clone,
        V: Clone,
    {
        map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }
}

/// Set operations
pub struct SetUtils;

impl SetUtils {
    /// Create new HashSet
    pub fn new<T>() -> std::collections::HashSet<T>
    where
        T: std::hash::Hash + Eq,
    {
        std::collections::HashSet::new()
    }

    /// Create with capacity
    pub fn with_capacity<T>(capacity: usize) -> std::collections::HashSet<T>
    where
        T: std::hash::Hash + Eq,
    {
        std::collections::HashSet::with_capacity(capacity)
    }

    /// Insert element
    pub fn insert<T>(set: &mut std::collections::HashSet<T>, item: T) -> bool
    where
        T: std::hash::Hash + Eq,
    {
        set.insert(item)
    }

    /// Check if contains element
    pub fn contains<T>(set: &std::collections::HashSet<T>, item: &T) -> bool
    where
        T: std::hash::Hash + Eq,
    {
        set.contains(item)
    }

    /// Remove element
    pub fn remove<T>(set: &mut std::collections::HashSet<T>, item: &T) -> bool
    where
        T: std::hash::Hash + Eq,
    {
        set.remove(item)
    }

    /// Get length
    pub fn len<T>(set: &std::collections::HashSet<T>) -> usize {
        set.len()
    }

    /// Check if empty
    pub fn is_empty<T>(set: &std::collections::HashSet<T>) -> bool {
        set.is_empty()
    }

    /// Clear set
    pub fn clear<T>(set: &mut std::collections::HashSet<T>)
    where
        T: std::hash::Hash + Eq,
    {
        set.clear();
    }

    /// Union of two sets
    pub fn union<T>(
        a: &std::collections::HashSet<T>,
        b: &std::collections::HashSet<T>,
    ) -> std::collections::HashSet<T>
    where
        T: std::hash::Hash + Eq + Clone,
    {
        a.union(b).cloned().collect()
    }

    /// Intersection of two sets
    pub fn intersection<T>(
        a: &std::collections::HashSet<T>,
        b: &std::collections::HashSet<T>,
    ) -> std::collections::HashSet<T>
    where
        T: std::hash::Hash + Eq + Clone,
    {
        a.intersection(b).cloned().collect()
    }

    /// Difference of two sets
    pub fn difference<T>(
        a: &std::collections::HashSet<T>,
        b: &std::collections::HashSet<T>,
    ) -> std::collections::HashSet<T>
    where
        T: std::hash::Hash + Eq + Clone,
    {
        a.difference(b).cloned().collect()
    }
}
