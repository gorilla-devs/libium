use std::collections::HashSet;

pub trait IterExt<T> {
    fn collect_vec(self) -> Vec<T>;
    fn collect_hashset(self) -> HashSet<T>
    where
        T: Eq + std::hash::Hash;
    /// Returns the indices of elements where `predicate` returns true
    fn positions(self, predicate: impl Fn(T) -> bool) -> impl Iterator<Item = usize>;
}

impl<T, I: Iterator<Item = T>> IterExt<T> for I {
    fn collect_vec(self) -> Vec<T> {
        self.collect::<Vec<T>>()
    }

    fn collect_hashset(self) -> HashSet<T>
    where
        T: Eq + std::hash::Hash,
    {
        self.collect::<HashSet<T>>()
    }

    fn positions(self, predicate: impl Fn(T) -> bool) -> impl Iterator<Item = usize> {
        self.enumerate()
            .filter_map(move |(i, e)| if predicate(e) { Some(i) } else { None })
    }
}

pub trait DisplayStrings {
    /// Delimits elements of `self` with a comma and returns a single string
    fn display(&self) -> String;
}

impl<S: ToString> DisplayStrings for Vec<S> {
    fn display(&self) -> String {
        self.iter()
            .map(ToString::to_string)
            .collect_vec()
            .join(", ")
    }
}
