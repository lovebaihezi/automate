use std::{
    collections::{BTreeMap, BTreeSet, HashSet, VecDeque},
    hash::Hash,
};

use super::StateMachine;
use crate::matches::Matcher;

/*
 * the NFA will have the multiply start point
 * and one start point is simple and manage
 * but we still can store the start point visited because
 * the start point is special and it will and always been used.
 */
type StateSet<T> = BTreeSet<T>;

#[derive(Debug)]
pub struct Nfa<S, V>
where
    S: Hash + Eq + Ord + Copy,
    V: Hash + Eq + Ord,
{
    pub start_state: S,
    end_state: HashSet<S>,
    pub all_state: HashSet<S>,
    pub all_path: BTreeSet<V>,
    maps: BTreeMap<S, BTreeMap<Option<V>, StateSet<S>>>,
}

impl<S, V> Nfa<S, V>
where
    S: Hash + Eq + Ord + Copy,
    V: Hash + Eq + Ord + Copy,
{
    #[inline]
    pub fn new(start_state: S) -> Self {
        Self {
            start_state,
            end_state: Default::default(),
            all_state: Default::default(),
            maps: Default::default(),
            all_path: Default::default(),
        }
    }
    #[inline]
    pub fn with_capacity(start_state: S, end_state_amount: usize, all_state_amount: usize) -> Self {
        Self {
            start_state,
            end_state: HashSet::with_capacity(end_state_amount),
            all_state: HashSet::with_capacity(all_state_amount),
            maps: BTreeMap::new(),
            all_path: Default::default(),
        }
    }
    #[inline]
    pub fn add_edges(&mut self, from: S, v: Option<V>, to: S) {
        if let Some(v) = v {
            self.all_path.insert(v);
        }
        if let Some(x) = self.maps.get_mut(&from) {
            if let Some(x) = x.get_mut(&v) {
                x.insert(to);
            } else {
                x.insert(v, [to].into());
            }
        } else {
            self.maps.insert(from, [(v, [to].into())].into());
        }
    }
    #[inline]
    pub fn add_states(&mut self, state: S) -> bool {
        self.all_state.insert(state)
    }
    #[inline]
    pub fn path_len(&self) -> usize {
        self.all_path.len()
    }
    #[inline]
    pub fn add_end_state(&mut self, node: S) -> bool {
        self.end_state.insert(node)
    }
    fn closure_recursion(&self, state: &S, previous: &mut HashSet<S>) {
        if let Some(map) = self.maps.get(state) {
            if let Some(set) = map.get(&None) {
                for i in set.iter() {
                    previous.insert(*i);
                    self.closure_recursion(i, previous);
                }
            }
        }
    }
    #[inline]
    pub fn closure(&self, state: &S) -> HashSet<S> {
        let mut set = HashSet::with_capacity(self.maps.len());
        let mut queue: VecDeque<&S> = [state].into();
        queue.reserve(self.all_state.len());
        while !queue.is_empty() {
            let top = queue.pop_back().unwrap();
            if !set.contains(top) {
                if let Some(map) = self.maps.get(top) {
                    if let Some(closure) = map.get(&None) {
                        queue.extend(closure.iter());
                        set.extend(closure.iter().copied());
                    }
                }
            }
        }
        set
    }
    #[inline]
    pub fn move_t(&self, state: &S, path: &V) -> HashSet<S> {
        let mut set = HashSet::new();
        if let Some(map) = self.maps.get(state) {
            if let Some(s) = map.get(&Some(*path)) {
                set.extend(s.iter());
            }
        }
        set
    }
}

impl<S, V> Nfa<S, V>
where
    S: Hash + Eq + Ord + Copy,
    V: Hash + Eq + Ord,
    Self: StateMachine<State = S, V = Option<V>, NextState = StateSet<S>>,
{
    #[inline]
    fn next_unvisited_state(
        &self,
        state: &S,
        v: &Option<V>,
        visited: &mut BTreeMap<S, BTreeSet<S>>,
    ) -> Option<S> {
        let get_next = |set: BTreeSet<_>| {
            visited.get_mut(state).and_then(|visit| {
                set.iter().fold(None, |s, v| {
                    s.or_else(|| if visit.contains(v) { s } else { Some(*v) })
                })
            })
        };
        self.next_state(state, v).and_then(get_next)
    }
}

impl<'a, S, V> StateMachine for &'a Nfa<S, V>
where
    S: Hash + Eq + Ord + Copy,
    V: Hash + Eq + Ord,
{
    type State = S;
    type V = Option<V>;
    type NextState = &'a StateSet<S>;
    #[inline]
    fn is_end(&self, state: &S) -> bool {
        self.end_state.get(state).is_some()
    }
    #[inline]
    fn next_state(&self, path: &Self::State, v: &Self::V) -> Option<Self::NextState> {
        if let Some(s) = self.maps.get(path) {
            s.get(v)
        } else {
            None
        }
    }
}

/*
 * if we use a set to store visited start points
 * so we needn't a initial state, we just choose a proper state
 * from NFA all start state, and process the DFS
 * this is kind of interesting and it's good for use, or we can
 * provide another API for a initial state start
 */
impl<S, V, I> Matcher<I> for Nfa<S, V>
where
    I: Iterator<Item = V>,
    S: Hash + Eq + Ord + Copy,
    V: Hash + Eq + Ord + Copy,
    Self: StateMachine<State = S, V = Option<V>, NextState = StateSet<S>>,
{
    type Matched = Vec<V>;
    #[inline]
    /*
     * Maybe we should use a Matches structure,
     * because as the iter is not reach its end, and we could find a path to reach the end
     * state, so we should continue NFA until the iter is end
     *
     *  make sure the iter will get correct length,or it will cost a long time on allocate memory
     *  space for Vec
     *
     */
    fn r#match(&self, iter: &mut I) -> Option<Self::Matched> {
        let mut stack: Vec<V> = Vec::with_capacity(match iter.size_hint() {
            (_, Some(x)) => x,
            (x, _) => x,
        });
        /*
         * what shall we store in the visited?
         * (state, v) => {state...}
         * when we back to previous state, we should choose another state to go,
         * so we should store current state visited state,if from the start state all have been
         * visited, go back to the previous until back to the start state, to access this control flow
         * 1. We can add a initial state, ** (initial state, EMPTY) => all start state ** and this
         *    must be done before the NFA is constructed
         * 2. For each all state in start state, if we find a road to the end state, the stack will
         *    contain the result we want, although the value store in will fallback to confirm, but
         *    all the v have already store in the maps, we could add it to the stack for the final
         *    result!
         */
        let mut visited: BTreeMap<S, BTreeSet<S>> = Default::default();
        let mut store: Vec<S> = Vec::with_capacity(self.all_state.len());
        let mut current_top = 0usize;
        store.push(self.start_state);
        /* most of time the S is always normal type like u32 or usize
         *
         * if we iterator all the v in the iterator,
         * and we can't reached the end state,we will get a None,
         * but if we reached a end state, then we can get the matched value in the iter
         *
         * but here comes questions:
         * 1. If from the current start state we can't reach the end state,shall we just pop
         *    the stack to a another stack? Or from the first of the stack and start check?
         *    which way?We already have multiply V store in the stack, and we should use that
         *    to get the next unique start state next to reach a end,if not,we shall return
         *    None to tell that current iter can't fit the NFA which means the iter can't match
         *    the Regular Expression
         */
        while !store.is_empty() {
            for _ in stack[current_top..].iter() {
                todo!()
            }
            for v in &mut *iter {
                if let Some(state) = store.last() {
                    if let Some(next_state) = self
                        .next_unvisited_state(state, &Some(v), &mut visited)
                        .or_else(|| self.next_unvisited_state(state, &None, &mut visited))
                    {
                        stack.push(v);
                        if self.is_end(&next_state) {
                            return Some(stack);
                            //question: if current is not equal to stack.len(),what shall we do?
                        } else {
                            current_top += 1;
                            visited.get_mut(state).map(|set| set.insert(next_state));
                            store.push(next_state);
                        }
                    } else {
                        todo!()
                    }
                } else {
                    break;
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod nfa_test {
    #[test]
    fn closure_calculate() {}
    #[test]
    fn nfa_match() {}
}
