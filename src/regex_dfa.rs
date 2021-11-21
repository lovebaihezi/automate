use crate::{dfa::Dfa, graph::Graph};
#[derive(Hash, Debug, PartialEq, Eq, Ord, PartialOrd, Clone)]
enum State {
    None,
}

pub struct Regex {
    dfa: Dfa<State, char>,
}

impl Regex {
    fn new(regex: &str) -> Self {
        todo!()
    }
    fn check(iter: impl Iterator<Item = char>) -> Vec<String> {
        todo!()
    }
}
