mod globals;
use globals::*;

mod agent;
pub use agent::Agent;

mod state_diff;
use state_diff::{FieldDiff, StateDiff};

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
    current_handle: usize,
}

impl Game {
    /*********        PUBLIC INTERFACES        *********/

    /// Return a new game.
    pub fn new(agents: Vec<Agent>) -> Self {
        let player_count = agents.len();
        Self {
            agents,
            move_history: vec![],
            state_nodes: vec![StateDiff::new_root(player_count)],
            dirty_states: vec![],
            current_handle: 0,
        }
    }

    /// Play the game until it ends.
    pub fn play(&mut self) {
        self.gen_children(self.current_state());
    }

    /*********        GETTERS        *********/

    /// Return an immutable reference to the current game state.
    fn current_state(&self) -> &StateDiff {
        &self.state_nodes[self.current_handle]
    }

    /*********        STATE PROPERTY GETTERS        *********/

    fn get_current_player(&self, state_handle: usize) -> &Player {
        &self.diff_players(state_handle)[self.diff_current_player(state_handle)]
    }

    /*********        STATE DIFF GETTERS        *********/

    /// Return a vector of players playing the game from `state`.
    fn diff_players(&self, state_handle: usize) -> &Vec<Player> {
        // Alias for the state in question
        let s = &self.state_nodes[state_handle];

        match s.get_diff_index(DIFF_ID_PLAYERS) {
            Some(i) => match &s.diffs[i] {
                FieldDiff::Players(p) => p,
                _ => unreachable!(),
            },
            // Look for `players` in the parent state if this state doesn't contain it
            None => self.diff_players(s.parent),
        }
    }

    /// Return the index of the player whose turn it currently is.
    fn diff_current_player(&self, state_handle: usize) -> usize {
        // Alias for the state in question
        let s = &self.state_nodes[state_handle];

        match s.get_diff_index(DIFF_ID_CURRENT_PLAYER) {
            Some(i) => match s.diffs[i] {
                FieldDiff::CurrentPlayer(p) => p,
                _ => unreachable!(),
            },
            // Look for `players` in the parent state if this state doesn't contain it
            None => self.diff_current_player(s.parent),
        }
    }

    /*********        STATE GENERATION        *********/

    /// Return child states that can be reached from the origin state.
    fn gen_children(&self, origin: &StateDiff) -> Vec<StateDiff> {
        self.gen_chance_children(origin)
    }

    /// Return child states that can be reached by rolling dice from this state.
    fn gen_chance_children(&self, origin: &StateDiff) -> Vec<StateDiff> {
        // self.diff_branch_type(origin);
        vec![]
    }
}
