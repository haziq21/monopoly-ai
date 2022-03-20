// TODO: Update `StateDiff`'s current_player everywhere it's needed.
// TODO: Optimise chance card branches.

use std::collections::HashMap;

mod globals;
use globals::*;

mod agent;
pub use agent::Agent;

mod state_diff;
use state_diff::{BranchType, FieldDiff, MoveType, PropertyOwnership, StateDiff};

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
    /*********       PUBLIC INTERFACES        *********/

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
        // TODO: Update parent state's children vector
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
        &self.diff_players(handle)[self.diff_current_pindex(handle)]
    }

    fn get_next_pindex(&self, handle: usize) -> usize {
        (self.diff_current_pindex(handle) + 1) % self.agents.len()
    }

    fn get_next_top_cc(&self, handle: usize) -> usize {
        (self.diff_top_cc(handle) + 1) % TOTAL_CHANCE_CARDS
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
    fn diff_current_pindex(&self, handle: usize) -> usize {
        // Alias for the state in question
        let s = &self.state_nodes[handle];

        match s.get_diff_index(DIFF_ID_CURRENT_PLAYER) {
            Some(i) => match s.diffs[i] {
                FieldDiff::CurrentPlayer(p) => p,
                _ => unreachable!(),
            },
            // Look for `current_players` in the parent state if this state doesn't contain it
            None => self.diff_current_pindex(s.parent),
        }
    }

    /// Return the properties that are owned by players at the specified state.
    fn diff_owned_properties(&self, handle: usize) -> &HashMap<u8, PropertyOwnership> {
        // Alias for the state in question
        let s = &self.state_nodes[handle];

        match s.get_diff_index(DIFF_ID_OWNED_PROPERTIES) {
            Some(i) => match &s.diffs[i] {
                FieldDiff::OwnedProperties(p) => p,
                _ => unreachable!(),
            },
            // Look for `owned_properties` in the parent state if this state doesn't contain it
            None => self.diff_owned_properties(s.parent),
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
            // Look for `seen_ccs` in the parent state if this state doesn't contain it
            None => self.diff_seen_ccs(s.parent),
        }
    }
    /// Return top_cc from the specified state.
    fn diff_top_cc(&self, handle: usize) -> usize {
        // Alias for the state in question
        let s = &self.state_nodes[handle];

        match s.get_diff_index(DIFF_ID_SEEN_CCS_HEAD) {
            Some(i) => match s.diffs[i] {
                FieldDiff::SeenCCsHead(p) => p,
                _ => unreachable!(),
            },
            // Look for `seen_ccs_head` in the parent state if this state doesn't contain it
            None => self.diff_top_cc(s.parent),
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
        // The index of the player whose turn it currently is
        let i = self.diff_current_pindex(handle);
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
                diff.set_branch_type(BranchType::Chance(roll.probability));
                // Clone the players
                let mut players = self.diff_players(handle).clone();
                // Update the current player's position
                players[i].move_by(roll.sum);

                // We didn't manage to roll doubles
                if !roll.is_double {
                    // $100 penalty for not rolling doubles
                    players[i].balance -= 100;
                }

                // Set the next move
                diff.next_move = MoveType::when_landed_on(players[i].position);
                // Set the players diff
                diff.set_players(players);
                // Update the current_player if needed
                if diff.next_move.is_roll() {
                    diff.set_current_pindex(self.get_next_pindex(handle));
                }

                children.push(diff);
            }
        }
        // Otherwise, play as normal
        else {
            // Loop through all possible dice results
            for roll in SIGNIFICANT_ROLLS.iter() {
                // Create a new state
                let mut state = StateDiff::new_with_parent(handle);
                // Update the branch type
                state.set_branch_type(BranchType::Chance(roll.probability));
                // Clone the players
                let mut players = self.diff_players(handle).clone();
                // Update the current player's position
                players[i].move_by(roll.sum);

                // Check if the player landed on 'go to jail'
                if players[i].position == GO_TO_JAIL_POSITION {
                    players[i].send_to_jail();
                }
                // Check if this roll got doubles
                else if roll.is_double {
                    // Increment the doubles_rolled counter
                    players[i].doubles_rolled += 1;

                    // Go to jail after three consecutive doubles
                    if players[i].doubles_rolled == 3 {
                        players[i].send_to_jail();
                    }
                } else {
                    // Reset the doubles counter
                    players[i].doubles_rolled = 0;
                }

                // Set the next move
                state.next_move = MoveType::when_landed_on(players[i].position);
                // Update the current_player if needed
                if state.next_move.is_roll() && players[i].doubles_rolled == 0 {
                    state.set_current_pindex(self.get_next_pindex(handle));
                }
                // Set the players diff
                state.set_players(players);

                children.push(state);
            }
        }

        children
    }

    /// Return child states that can be reached by picking a chance card from the specified state.
    fn gen_cc_children(&self, handle: usize) -> Vec<StateDiff> {
        let mut children = vec![];
        let seen_ccs = self.diff_seen_ccs(handle);

        // We can deduce the exact chance card that we're going to get since we've seen them all
        if seen_ccs.len() == TOTAL_CHANCE_CARDS {
            // The chance card that the player will definitely get
            let definite_cc = seen_ccs[self.diff_top_cc(handle)];

            // Get the child diffs according to the choicefulness of the chance card
            if definite_cc.is_choiceless() {
                // This is the only possibility since this is a choiceless chance card
                return vec![self.gen_choiceless_cc(definite_cc, handle, 1.)];
            }

            return self.gen_choiceful_cc_children(handle, definite_cc);
        }

        // We can't know the exact chance card that we're
        // going to get, so calculate all their probabilities
        let unseen_cards = ChanceCard::unseen_counts(&seen_ccs);

        for (card, count) in unseen_cards {
            // Calculate the probability of encountering this chance card
            let probability = count as f64 / (TOTAL_CHANCE_CARDS - seen_ccs.len()) as f64;

            // Skip if the chance card has no chance of occurring
            if probability == 0. {
                continue;
            }

            if card.is_choiceless() {
                children.push(self.gen_choiceless_cc(card, handle, probability));
            } else {
                let mut state = StateDiff::new_with_parent(handle);
                state.set_branch_type(BranchType::Chance(probability));
                state.next_move = MoveType::ChoicefulCC(card);
                children.push(state);
            };
        }

        children
    }

    /*********        CHOICEFUL CC STATE GENERATION        *********/

    /// Return child states that can be reached by getting the
    fn gen_choiceful_cc_children(&self, handle: usize, cc: ChanceCard) -> Vec<StateDiff> {
        match cc {
            ChanceCard::RentTo5 => self.gen_cc_rent_to_x(true, handle),
            ChanceCard::RentTo1 => self.gen_cc_rent_to_x(false, handle),
            ChanceCard::SetRentInc => self.gen_cc_set_rent_change(true, handle),
            ChanceCard::SetRentDec => self.gen_cc_set_rent_change(false, handle),
            ChanceCard::SideRentInc => self.gen_cc_side_rent_change(true, handle),
            ChanceCard::SideRentDec => self.gen_cc_side_rent_change(false, handle),
            _ => unimplemented!(),
        }
    }

    /// 'RentToX'  chance card. Return a vector of all possible choice effects.
    fn gen_cc_rent_to_x(&self, max: bool, handle: usize) -> Vec<StateDiff> {
        let mut children = vec![];
        let curr_pindex = self.diff_current_pindex(handle);
        let (cc, target_rent) = if max {
            (ChanceCard::RentTo1, 1)
        } else {
            (ChanceCard::RentTo5, 5)
        };

        for (pos, prop) in self.diff_owned_properties(handle) {
            // "RentTo5" only applies to your properties (not opponents), and we don't
            // need to add another child node if the rent level is already at its max/min
            if max && prop.owner != curr_pindex || prop.rent_level == target_rent {
                continue;
            }

            // Create the diff
            let mut child = self.new_state_from_cc(cc, handle);
            child.set_branch_type(BranchType::Choice);
            // Update the owned_properties
            let mut owned_props = self.diff_owned_properties(handle).clone();
            owned_props.get_mut(&pos).unwrap().rent_level = target_rent;
            child.set_owned_properties(owned_props);

            children.push(child);
        }

        children
    }

    fn gen_cc_set_rent_change(&self, increase: bool, handle: usize) -> Vec<StateDiff> {
        let mut children = vec![];
        let cc = if increase {
            ChanceCard::SetRentInc
        } else {
            ChanceCard::SetRentDec
        };

        // Loop through each color set
        for (_, positions) in PROPS_BY_COLOR.iter() {
            let mut new_state = self.new_state_from_cc(cc, handle);
            let mut owned_props = self.diff_owned_properties(handle).clone();
            let mut has_effect = false;
            // Loop through all the properties in this color set
            for pos in positions {
                // Check if a property exists at `pos`
                if let Some(prop) = owned_props.get_mut(&pos) {
                    has_effect |= prop.change_rent(increase);
                }
            }
            // Only store the new state if it's different
            if has_effect {
                new_state.set_branch_type(BranchType::Choice);
                new_state.set_owned_properties(owned_props);
                children.push(new_state);
            }
        }

        children
    }

    fn gen_cc_side_rent_change(&self, increase: bool, handle: usize) -> Vec<StateDiff> {
        let mut children = vec![];
        let cc = if increase {
            ChanceCard::SideRentInc
        } else {
            ChanceCard::SideRentDec
        };

        for positions in PROPS_BY_SIDE.iter() {
            let mut child = self.new_state_from_cc(cc, handle);
            let mut owned_properties = self.diff_owned_properties(handle).clone();
            let mut has_effect = false;

            for pos in positions {
                // Check if the property is owned
                if let Some(prop) = owned_properties.get_mut(&pos) {
                    has_effect |= prop.change_rent(increase);
                }
            }

            // Save the child if it's different
            if has_effect {
                child.set_branch_type(BranchType::Choice);
                child.set_owned_properties(owned_properties);
                children.push(child);
            }
        }

        children
    }

    fn gen_cc_rent_spike(&self, handle: usize) -> Vec<StateDiff> {
        vec![]
    }
    /*********        CHOICELESS CC STATE GENERATION        *********/

    /// Modify `state` according to the effects of the `cc` chance card.
    /// `parent_handle` is the handle of `state`'s parent.
    fn gen_choiceless_cc(&self, cc: ChanceCard, handle: usize, probability: f64) -> StateDiff {
        match cc {
            ChanceCard::PropertyTax => self.gen_cc_property_tax(probability, handle),
            ChanceCard::Level1Rent => self.gen_cc_level_1_rent(probability, handle),
            ChanceCard::AllToParking => self.gen_cc_all_to_parking(probability, handle),
            _ => panic!("choiceful cc passed to Game.mod_choiceless_cc()"),
        }
    }

    /// Modify `state` according to the effects of the 'property tax'
    /// chance card. `parent_handle` is the handle of `state`'s parent.
    fn gen_cc_property_tax(&self, probability: f64, handle: usize) -> StateDiff {
        let mut tax = 0;
        let i = self.diff_current_pindex(handle);

        // Tax $50 per property owned
        for (_, prop) in self.diff_owned_properties(handle) {
            if prop.owner == i {
                tax += 50;
            }
        }

        // Clone the players
        let mut updated_players = self.diff_players(handle).clone();
        // Update the players based on the calculated tax
        updated_players[i].balance -= tax;

        // Create a new state
        let mut state = self.new_state_from_cc(ChanceCard::PropertyTax, handle);
        state.set_branch_type(BranchType::Chance(probability));
        state.set_players(updated_players);

        state
    }

    /// Modify `state` according to the effects of the 'level 1 rent' chance card.
    fn gen_cc_level_1_rent(&self, probability: f64, handle: usize) -> StateDiff {
        let mut state = self.new_state_from_cc(ChanceCard::Level1Rent, handle);
        state.set_branch_type(BranchType::Chance(probability));
        // Set the diff to 2 rounds (player_count * 2 turns per player)
        state.set_level_1_rent(self.agents.len() as u8 * 2);

        state
    }

    /// Modify `state` according to the effects of the 'all to parking'
    /// chance card. `parent_handle` is the handle of `state`'s parent.
    fn gen_cc_all_to_parking(&self, probability: f64, handle: usize) -> StateDiff {
        // Clone players
        let mut updated_players = self.diff_players(handle).clone();

        // Move every player who's not in jail to free parking
        for player in &mut updated_players {
            if !player.in_jail {
                player.position = FREE_PARKING_POSITION;
            }
        }

        // Create a new state
        let mut state = self.new_state_from_cc(ChanceCard::AllToParking, handle);
        state.set_branch_type(BranchType::Chance(probability));
        state.set_players(updated_players);

        state
    }

    /// - Set `next_move` to `Roll`
    /// - Update `current_player` if needed
    /// - Update `seen_ccs_head` if needed
    fn new_state_from_cc(&self, card: ChanceCard, handle: usize) -> StateDiff {
        let mut state = StateDiff::new_with_parent(handle);
        state.next_move = MoveType::Roll;

        // It's the next player's turn if the current player didn't roll doubles
        if self.current_player(handle).doubles_rolled == 0 {
            state.set_current_pindex(self.get_next_pindex(handle));
        }

        // Update the top_cc if needed
        if self.diff_seen_ccs(handle).len() == TOTAL_CHANCE_CARDS {
            state.set_top_cc(self.get_next_top_cc(handle));
        } else {
            let mut seen_ccs = self.diff_seen_ccs(handle).clone();
            seen_ccs.push(card);
            state.set_seen_ccs(seen_ccs);
        }

        state
    }
}
