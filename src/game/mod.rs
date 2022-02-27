mod globals;
use globals::*;

mod agent;
pub use agent::Agent;

mod state_diff;
use state_diff::{BranchType, FieldDiff, StateDiff};

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
        self.gen_children(self.current_handle);
    }

    /*********        HELPERS        *********/

    /// Push the new state node to `self.state_nodes` and return its handle.
    // fn append_state(&mut self, )

    /*********        STATE PROPERTY GETTERS        *********/

    /// Return the player whose turn it currently is at the specified state.
    fn current_player(&self, handle: usize) -> &Player {
        &self.diff_players(handle)[self.diff_current_player(handle)]
    }

    /*********        STATE DIFF GETTERS        *********/

    /// Return a vector of players playing the game at the specified state.
    fn diff_players(&self, handle: usize) -> &Vec<Player> {
        // Alias for the state in question
        let s = &self.state_nodes[handle];

        match s.get_diff_index(DIFF_ID_PLAYERS) {
            Some(i) => match &s.diffs[i] {
                FieldDiff::Players(p) => p,
                _ => unreachable!(),
            },
            // Look for `players` in the parent state if this state doesn't contain it
            None => self.diff_players(s.parent),
        }
    }

    /// Return the index of the player whose turn it currently is at the specified state.
    fn diff_current_player(&self, handle: usize) -> usize {
        // Alias for the state in question
        let s = &self.state_nodes[handle];

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

    /// Return child states that can be reached from the specified state.
    fn gen_children(&self, handle: usize) -> Vec<StateDiff> {
        self.gen_chance_children(handle)
    }

    /// Return child states that can be reached by rolling dice from the specified state.
    fn gen_chance_children(&self, handle: usize) -> Vec<StateDiff> {
        let current_player_index = self.diff_current_player(handle);
        let mut children = vec![];

        // Get the player out of jail if they're in jail
        if self.current_player(handle).in_jail {
            // Try rolling doubles to get out of jail
            let double_probabilities = roll_for_doubles(3);

            // Loop through all possible dice results
            for roll in double_probabilities {
                // Create a new diff
                let mut diff = StateDiff::new_with_parent(handle);
                // Update the branch type
                diff.set_branch_type_diff(BranchType::Chance(roll.probability));
                // Clone the players
                let players = diff.set_players_diff(self.diff_players(handle).clone());

                // We didn't manage to roll doubles
                if !roll.is_double {
                    // $100 penalty for not rolling doubles
                    players[current_player_index].balance -= 100;
                }

                // Update the current player's position
                players[current_player_index].move_by(roll.sum);
                children.push(diff);
            }
        }
        // Otherwise, play as normal
        else {
            // Loop through all possible dice results
            for roll in SIGNIFICANT_ROLLS.iter() {
                // Create a new diff
                let mut diff = StateDiff::new_with_parent(handle);
                // Update the branch type
                diff.set_branch_type_diff(BranchType::Chance(roll.probability));
                // Clone the players
                let players = diff.set_players_diff(self.diff_players(handle).clone());
                // Alias for the player whose turn it currently is
                let curr_player = &mut players[current_player_index];

                // Update the current player's position
                curr_player.move_by(roll.sum);

                // Check if the player landed on 'go to jail'
                if curr_player.position == 27 {
                    curr_player.send_to_jail();
                }
                // Check if this roll got doubles
                else if roll.is_double {
                    // Increment the doubles_rolled counter
                    curr_player.doubles_rolled += 1;

                    // Go to jail after three consecutive doubles
                    if curr_player.doubles_rolled == 3 {
                        curr_player.send_to_jail();
                    }
                } else {
                    // Reset the doubles counter
                    curr_player.doubles_rolled = 0;
                }

                children.push(diff);
            }
        }

        children
    }
}
