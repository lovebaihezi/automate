use core::cmp::Ordering;
use core::hash::Hash;
#[derive(Debug, Eq, Copy, Clone)]
pub enum Action<V>
where
    V: Hash + Eq + Ord + Copy,
{
    Range(V, V),
    Single(V),
}

impl<V> Hash for Action<V>
where
    V: Hash + Eq + Ord + Copy,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl<V> PartialEq for Action<V>
where
    V: Hash + Eq + Ord + Copy,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Action::Range(s1, s2), Action::Range(v1, v2)) => s1 == v1 && s2 == v2,
            (Action::Single(v1), Action::Single(v2)) => v1 == v2,
            (Action::Range(s1, s2), Action::Single(c)) => s1 <= c && c <= s2,
            (this, other) => other.eq(this),
        }
    }
}

impl<V> PartialEq<V> for Action<V>
where
    V: Hash + Eq + Ord + Copy,
{
    fn eq(&self, c: &V) -> bool {
        match self {
            Action::Range(s, e) => s <= c && c <= e,
            Action::Single(v) => v == c,
        }
    }
}

impl<V> PartialOrd for Action<V>
where
    V: Hash + Eq + Ord + Copy,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Action::Range(v1, v2), Action::Range(s1, s2)) => {
                assert!(v2 > v1 && s2 > s1);
                Some(if v1 == s1 && v2 == s2 {
                    Ordering::Equal
                } else if v2 > s2 {
                    Ordering::Greater
                } else {
                    Ordering::Less
                })
            }
            (Action::Range(s, e), Action::Single(c)) => Some(if s <= c && c <= e {
                Ordering::Equal
            } else if c > e {
                Ordering::Greater
            } else {
                Ordering::Less
            }),
            (Action::Single(v1), Action::Single(v2)) => v1.partial_cmp(v2),
            (v, s) => s.partial_cmp(v),
        }
    }
}

impl<V> PartialOrd<V> for Action<V>
where
    V: Hash + Eq + Ord + Copy,
{
    fn partial_cmp(&self, other: &V) -> Option<Ordering> {
        match self {
            Action::Range(s, e) => Some({
                if other >= s && other <= e {
                    Ordering::Equal
                } else if other > e {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            }),
            Action::Single(v) => v.partial_cmp(other),
        }
    }
}

impl<V> Ord for Action<V>
where
    V: Hash + Eq + Ord + Copy,
{
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Action::Range(v1, v2), Action::Range(s1, s2)) => {
                assert!(v2 > v1 && s2 > s1);
                if v1 == s1 && v2 == s2 {
                    Ordering::Equal
                } else if v2 > s2 {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            }
            (Action::Range(s, e), Action::Single(c)) => {
                if s <= c && c <= e {
                    Ordering::Equal
                } else if c > e {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            }
            (Action::Single(v1), Action::Single(v2)) => v1.cmp(v2),
            (v, s) => s.cmp(v),
        }
    }
}
#[cfg(test)]
mod action_test {}
