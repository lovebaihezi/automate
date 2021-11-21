pub trait StateMachine {
    type State;
    type V;
    type NextState;
    fn is_end(&self, state: &Self::State) -> bool;
    fn next_state(&self, from: &Self::State, path: &Self::V) -> Option<Self::NextState>;
}
