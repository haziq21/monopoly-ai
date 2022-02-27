mod globals;
use globals::*;

mod agent;
pub use agent::Agent;

mod state_diff;
use state_diff::{BranchType, FieldDiff, MoveType, StateDiff};

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
    dirty_handles: Vec<usize>,
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
            dirty_handles: vec![],
            current_handle: 0,
        }
    }

    /// Play the game until it ends.
    pub fn play(&mut self) {
        for state in self.gen_children(self.current_handle) {
            let handle = self.append_state(state);
            self.gen_children(handle);
        }
    }

    /*********        HELPERS        *********/

    /// Push the new state node to `self.state_nodes` and return its handle.
    fn append_state(&mut self, state: StateDiff) -> usize {
        if self.dirty_handles.len() == 0 {
            self.state_nodes.push(state);
            return self.state_nodes.len() - 1;
        }

        let i = self.dirty_handles[0];
        self.state_nodes[i] = state;
        self.dirty_handles.swap_remove(0);

        i
    }

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

    /// Return a vector of chance cards that have already been seen from the specified state.
    fn diff_seen_ccs(&self, handle: usize) -> &Vec<ChanceCard> {
        // Alias for the state in question
        let s = &self.state_nodes[handle];

        match s.get_diff_index(DIFF_ID_SEEN_CCS) {
            Some(i) => match &s.diffs[i] {
                FieldDiff::SeenCCs(p) => p,
                _ => unreachable!(),
            },
            // Look for `players` in the parent state if this state doesn't contain it
            None => self.diff_seen_ccs(s.parent),
        }
    }

    /*********        STATE GENERATION        *********/

    /// Return child states that can be reached from the specified state.
    fn gen_children(&self, handle: usize) -> Vec<StateDiff> {
        match self.state_nodes[handle].next_move {
            MoveType::Roll => self.gen_roll_children(handle),
            MoveType::ChanceCard => self.gen_cc_children(handle),
            MoveType::Undefined => unreachable!(),
            _ => unimplemented!(),
        }
    }

    /// Return child states that can be reached by rolling dice from the specified state.
    fn gen_roll_children(&self, handle: usize) -> Vec<StateDiff> {
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
                // Set the next move
                diff.next_move = MoveType::when_landed_on(players[current_player_index].position);
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
                if curr_player.position == GO_TO_JAIL_POSITION {
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

                // Set the next move
                diff.next_move = MoveType::when_landed_on(curr_player.position);
                children.push(diff);
            }
        }

        children
    }

    /// Return child states that can be reached by picking a chance card from the specified state.
    fn gen_cc_children(&self, handle: usize) -> Vec<StateDiff> {
        let mut children = vec![];
        let seen_ccs = self.diff_seen_ccs(handle);

        if seen_ccs.len() == 21 {}

        children
    }
}
