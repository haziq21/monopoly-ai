mod globals;

mod agent;
pub use agent::Agent;

mod state_diff;
use state_diff::StateDiff;

/// A simulation of Monopoly.
pub struct Game {
    /// Agents playing the game.
    agents: Vec<Agent>,
    /// The moves taken by players in terms of the indexes of the children.
    move_history: Vec<usize>,
    /// The current game state, as well as all its decendants.
    state_nodes: Vec<StateDiff>,
    /// Indexes of states that have been marked for deletion.
    /// These states can be safely replaced by newer states.
    dirty_states: Vec<usize>,
    /// The index of the state the game is currently at.
    current_state: usize,
}

impl Game {
    /*********        PUBLIC INTERFACES        *********/

    /// Return a new game.
    pub fn new(agents: Vec<Agent>) -> Self {
        let player_count = agents.len();
        Self {
            agents,
            move_history: vec![],
            state_nodes: vec![StateDiff::new(player_count)],
            dirty_states: vec![],
            current_state: 0,
        }
    }

    /// Play the game until it ends.
    pub fn play(&mut self) {
        // Placeholder
    }
}
