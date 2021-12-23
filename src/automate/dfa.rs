use super::Nfa;
use super::StateMachine;
use crate::matches::Matcher;

use rayon::prelude::*;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
    fmt::{Debug, Display},
    hash::Hash,
};

#[derive(Debug)]
pub enum DfaError {
    InvalidRelationship,
}

/*
 * @Author: lqxc,
 * Now we have a question about if the regular accept a range of values,which we have to add many
 * node into the RB-Tree,with a small range,it wouldn't cost a lot of memory to store a
 * range,further more,the user of the regex just describe a Regular Expression and use the dfa to
 * match a string,they need not to actually call match with parameter,but only the V they send is
 * lack of information of which range will the V fitted,but maybe this is not very important now,as
 * the complex of the dfa growth by the concentrating of the performance,we shall handle it by
 * match or if-else not the through graph to handle this complexity,and macro or compile a source
 * for lexical will be more fast then the graph
 */

#[derive(Debug)]
pub struct Dfa<S, V>
where
    S: Hash + Eq + Ord,
    V: Hash + Eq + Ord,
{
    start_state: S,
    end_state: HashSet<S>,
    all_state: HashSet<S>,
    maped: BTreeMap<S, BTreeMap<V, S>>,
}

impl<S, V> Dfa<S, V>
where
    S: Hash + Eq + Ord + Copy,
    V: Hash + Eq + Ord + Copy,
{
    #[inline]
    pub fn new(start_state: S) -> Self {
        Self {
            start_state,
            end_state: HashSet::new(),
            all_state: Default::default(),
            maped: Default::default(),
        }
    }
    #[inline]
    pub fn with_capacity(start_state: S, end_state_amount: usize, all_state_amount: usize) -> Self {
        Self {
            start_state,
            end_state: HashSet::with_capacity(end_state_amount),
            all_state: HashSet::with_capacity(all_state_amount),
            maped: Default::default(),
        }
    }
    #[inline]
    pub fn add_edges(&mut self, from: S, v: V, to: S) -> Result<(), DfaError> {
        if let Some(map) = self.maped.get_mut(&from) {
            if map.get(&v).is_some() {
                Err(DfaError::InvalidRelationship)
            } else {
                map.insert(v, to);
                Ok(())
            }
        } else {
            self.maped.insert(from, [(v, to)].into_iter().collect());
            Ok(())
        }
    }
    #[inline]
    pub fn add_pattern(&mut self, from: S, v: &[V], to: S) -> Result<(), DfaError> {
        for v in v {
            self.add_edges(from, *v, to)?;
        }
        Ok(())
    }
    #[inline]
    pub fn add_end_state(&mut self, node: S) -> bool {
        self.end_state.insert(node)
    }
    #[inline]
    pub fn optimize(&mut self) {
        let mut other_state: BTreeSet<S> = self.end_state.union(&self.all_state).copied().collect();
        // let mut result = HashSet::with_capacity(self.all_state.len());
        // while !other_state.is_empty() && !self.end_state.is_empty() {}
        // for i in 0..result.len() {}
    }
}

fn closure<'a, S: 'a, V>(nfa: &Nfa<S, V>, set: &mut impl Iterator<Item = &'a S>) -> BTreeSet<S>
where
    S: Hash + Ord + Copy + Eq,
    V: Hash + Ord + Copy + Eq,
{
    let mut result = BTreeSet::new();
    set.map(|state| nfa.closure(state)).for_each(|set| {
        for state in set.iter() {
            result.insert(*state);
        }
    });
    result
}

fn move_t<'a, S: 'a, V>(
    nfa: &Nfa<S, V>,
    set: &mut impl Iterator<Item = &'a S>,
    path: &V,
) -> BTreeSet<S>
where
    S: Hash + Ord + Copy + Eq,
    V: Hash + Ord + Copy + Eq,
{
    let mut result = BTreeSet::new();
    set.map(|state| nfa.move_t(state, path)).for_each(|set| {
        for state in set.into_iter() {
            result.insert(state);
        }
    });
    result
}

impl<S, V> FromIterator<(S, V, S)> for Dfa<S, V>
where
    S: Hash + Eq + Ord + Copy + Default,
    V: Hash + Eq + Ord + Copy,
{
    fn from_iter<T: IntoIterator<Item = (S, V, S)>>(iter: T) -> Self {
        let mut dfa = Dfa::new(Default::default());
        iter.into_iter()
            .for_each(|(from, path, to)| dfa.add_edges(from, path, to).unwrap());
        dfa
    }
}

impl<'a, S, V> From<&'a Nfa<S, V>> for Dfa<usize, V>
where
    S: Hash + Ord + Eq + Copy + Default,
    V: Hash + Ord + Eq + Copy,
{
    fn from(nfa: &'a Nfa<S, V>) -> Self {
        let init: Vec<S> = closure(nfa, &mut [nfa.start_state].iter())
            .into_iter()
            .collect();
        let mut map: HashMap<Vec<S>, usize> = [(init.clone(), 0)].into();
        let mut v = vec![(init, BTreeMap::new())];
        let mut top = 0usize;
        let mut store = Vec::new();
        while top != v.len() {
            let c = top;
            top = v.len();
            for (i, (new_state, path_map)) in v[c..].iter_mut().enumerate() {
                let closure = closure(nfa, &mut new_state.iter());
                for path in nfa.all_path.iter() {
                    let new_set: Vec<S> = closure
                        .iter()
                        .copied()
                        .chain(move_t(nfa, &mut closure.iter(), path).into_iter())
                        .collect();
                    if !new_set.is_empty() {
                        if !map.contains_key(&new_set) {
                            map.insert(new_set.clone(), i + c + 1);
                            store.push((new_set.clone(), Default::default()));
                        }
                        path_map.insert(path, new_set);
                    }
                }
            }
            v.append(&mut store);
        }
        /*
         * Here comes a question,if the DFA have multiple start state,then the DFA will become much
         * slower then we want O(|s|) -> O(|r * S|).
         */
        let mut dfa = Dfa::new(Default::default());
        for (state, path_map) in v {
            let index = map.get(&state).unwrap();
            for i in state {
                if nfa.is_end(&i) {
                    dfa.add_end_state(*index);
                }
            }
            for (key, value) in path_map {
                dfa.add_edges(*index, *key, *map.get(&value).unwrap())
                    .unwrap();
            }
        }
        dfa
    }
}

impl<S, V> StateMachine for Dfa<S, V>
where
    S: Hash + Eq + Ord + Copy,
    V: Hash + Eq + Ord + Copy,
{
    type State = S;

    type V = V;

    type NextState = S;

    fn is_end(&self, state: &Self::State) -> bool {
        self.end_state.contains(state)
    }

    fn next_state(&self, from: &Self::State, path: &Self::V) -> Option<Self::NextState> {
        self.maped
            .get(from)
            .and_then(|tree| tree.get(path).copied())
    }
}

impl<S, V, I> Matcher<I> for Dfa<S, V>
where
    S: Hash + Eq + Ord + Copy + Debug,
    V: Hash + Eq + Ord + Copy + Debug,
    I: Iterator<Item = V>,
    Self: StateMachine<NextState = S, State = S, V = V>,
{
    type Matched = Vec<V>;

    fn r#match(&self, iter: &mut I) -> Option<Self::Matched> {
        let mut state = self.start_state;
        let mut result = if let (_, Some(l)) = iter.size_hint() {
            Vec::with_capacity(l)
        } else {
            Vec::with_capacity(iter.size_hint().0)
        };
        for i in &mut *iter {
            match self.next_state(&state, &i) {
                Some(next_state) => {
                    result.push(i);
                    state = next_state;
                }
                None => {
                    if result.is_empty() {
                        continue;
                    } else {
                        return if self.is_end(&state) {
                            Some(result)
                        } else {
                            None
                        };
                    }
                }
            }
        }
        None
    }
}

#[macro_export]
macro_rules! Dfa {
    {
        Start:$start:expr,
        End:[$($end:expr),+$(,)*],
        V: {$(
            $from:expr =>($v:expr)=> $to:expr,
        )+}
    }=> {
        {
            let mut dfa = $crate::automate::Dfa::new($start);
            $(dfa.add_end_state($end);)+
            $(dfa.add_edges($from, $v, $to).unwrap();)*
            dfa
        }
    };
}

#[cfg(test)]
mod test_dfa {
    use super::super::Action;
    use crate::matches::Matcher;
    #[test]
    fn dfa_macro() {
        let dfa = Dfa! {
            Start:1,
            End:[2, 1],
            V: {
                1 => (Action::Single('b')) => 1,
                1 => (Action::Range('d', 'y')) => 3,
                3 => (Action::Single('1')) => 1,
                3 => (Action::Single('2')) => 1,
            }
        };
        let mut iter = "123456".chars().map(Action::Single);
        assert_eq!(
            dfa.matches(&mut iter).collect::<Vec<Vec<Action<char>>>>(),
            Vec::<Vec<Action<char>>>::new()
        );
    }
    #[test]
    fn determine_nfa() {}
}
