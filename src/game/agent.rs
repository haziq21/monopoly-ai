use super::state::State;

pub enum Agent {
    Ai,
    Human,
}

impl Agent {
    pub fn make_choice(&self, from_node: State) -> State {
        match self {
            Agent::Ai => Agent::ai_choice(from_node),
            Agent::Human => Agent::human_choice(from_node),
        }
    }

    pub fn ai_choice(from_node: State) -> State {
        State::new(2)
    }

    pub fn human_choice(from_node: State) -> State {
        State::new(2)
    }
}
