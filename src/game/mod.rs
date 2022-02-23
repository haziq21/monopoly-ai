mod globals;

mod agent;
pub use agent::Agent;

mod state;
use state::StateDiff;

pub struct Game {
    pub agents: Vec<Agent>,
    pub current_state: usize,
    pub move_history: Vec<usize>,
    child_diffs: Vec<StateDiff>,
}

impl Game {
    /*********        PUBLIC INTERFACES        *********/

    /// Create a new game.
    pub fn new(agents: Vec<Agent>) -> Self {
        let player_count = agents.len();
        Self {
            agents,
            current_state: 0,
            child_diffs: vec![StateDiff::new(player_count)],
            move_history: vec![],
        }
    }

    /// Play the game until it ends.
    pub fn play(&mut self) {
        // Placeholder
        // self.agents[0].make_choice(&mut self.current_state, &self.move_history);
    }
}
