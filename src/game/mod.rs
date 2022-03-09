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
            // Look for `current_players` in the parent state if this state doesn't contain it
            None => self.diff_current_player(s.parent),
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
    /// Return seen_ccs_head from the specified state.
    fn diff_seen_ccs_head(&self, handle: usize) -> usize {
        // Alias for the state in question
        let s = &self.state_nodes[handle];

        match s.get_diff_index(DIFF_ID_SEEN_CCS_HEAD) {
            Some(i) => match s.diffs[i] {
                FieldDiff::SeenCCsHead(p) => p,
                _ => unreachable!(),
            },
            // Look for `seen_ccs_head` in the parent state if this state doesn't contain it
            None => self.diff_seen_ccs_head(s.parent),
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

        // We can deduce the exact chance card that we're going to get since we've seen them all
        if seen_ccs.len() == 21 {
            // The chance card that the player will definitely get
            let definite_cc = seen_ccs[self.diff_seen_ccs_head(handle)];

            // Get the child diffs according to the choicefulness of the chance card
            if definite_cc.is_choiceless() {
                // Create a template for the state diff
                let mut template_diff = StateDiff::new_with_parent(handle);
                // 100% chance of getting this card
                template_diff.set_branch_type_diff(BranchType::Chance(1.));
                // Apply the chance card's effects onto the template diff
                self.mod_choiceless_cc(&mut template_diff, definite_cc, handle);
                // template_diff is the only possibility since this is a choiceless chance card
                return vec![template_diff];
            }

            return self.gen_choiceful_cc_children(handle, definite_cc);
        }

        // We can't know the exact chance card that we're
        // going to get, so calculate all their probabilities
        let unseen_cards = ChanceCard::unseen_counts(&seen_ccs);

        for (card, count) in unseen_cards {
            // Calculate the probability of encountering this chance card
            let probability = count as f64 / (21 - seen_ccs.len()) as f64;

            // Skip if the chance card has no chance of occurring
            if probability == 0. {
                continue;
            }

            // Create a child state
            let mut diff = StateDiff::new_with_parent(handle);
            // The state was reached by chance (getting this chance card by chance)
            diff.set_branch_type_diff(BranchType::Chance(probability));

            if card.is_choiceless() {
                // If the chance card is a choiceless one, then the move will be over over once the
                // chance card's effects are applied and it will be the next person's turn to roll dice
                diff.next_move = MoveType::Roll;
                self.mod_choiceless_cc(&mut diff, card, handle);
            } else {
                // If the chance card is a choiceful one, then the next move is to
                // make a choice according to the chance card, so we reset the `next_move`
                diff.next_move = MoveType::ChoicefulCC(card);
            }

            children.push(diff);
        }

        children
    }

    fn gen_choiceful_cc_children(&self, handle: usize, cc: ChanceCard) -> Vec<StateDiff> {
        match cc {
            ChanceCard::RentTo1 => self.gen_cc_rent_to_x(1, handle),
            ChanceCard::RentTo5 => self.gen_cc_rent_to_x(5, handle),
            ChanceCard::SetRentInc => self.gen_cc_set_rent_change(true, handle),
            _ => unimplemented!(),
        }
    }

    /// Return child states that can be reached by getting the
    /// 'RentToX'  chance card. Return a vector of all possible choice effects.
    fn gen_cc_rent_to_x(&self, x: u8, handle: usize) -> Vec<StateDiff> {
        let mut children = vec![];
        let curr_player = self.diff_current_player(handle);
        let cc = if x == 1 {
            ChanceCard::RentTo1
        } else {
            ChanceCard::RentTo5
        };

        for (pos, prop) in self.diff_owned_properties(handle) {
            // "RentTo5" only applies to your properties (not opponents), and we don't
            // need to add another child node if the rent level is already at its max/min
            if x == 5 && prop.owner != curr_player || prop.rent_level == x {
                continue;
            }

            // Create the diff
            let mut child = StateDiff::new_with_parent(handle);
            // Clone the owned_properties
            let mut cloned_owned_props = self.diff_owned_properties(handle).clone();
            // Update the owned_properties
            cloned_owned_props.get_mut(&pos).unwrap().rent_level = x;
            // Set the diff
            child.set_owned_properties_diff(cloned_owned_props);
            // Apply the boilerplate
            self.add_cc_boilerplate(cc, &mut child, handle);

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
            let mut new_state = StateDiff::new_with_parent(handle);
            let mut owned_props = self.diff_owned_properties(handle).clone();
            let mut has_effect = false;

            // Loop through all the properties in this color set
            for pos in positions {
                // Check if a property exists at `pos`
                if let Some(prop) = owned_props.get_mut(&pos) {
                    has_effect |= if increase {
                        prop.raise_rent()
                    } else {
                        prop.lower_rent()
                    }
                }
            }

            // Only store the new state if it's different
            if has_effect {
                new_state.set_owned_properties_diff(owned_props);
                self.add_cc_boilerplate(cc, &mut new_state, handle);
                children.push(new_state);
            }
        }

        children
    }

    // fn gen_cc_side_rent_change(&self, increase: bool, handle: usize) -> Vec<StateDiff> {}

    /*********        CHOICELESS CC STATE MODIFICATION        *********/

    /// Modify `state` according to the effects of the `cc` chance card.
    /// `parent_handle` is the handle of `state`'s parent.
    fn mod_choiceless_cc(&self, state: &mut StateDiff, cc: ChanceCard, handle: usize) {
        // Apply the boilerplate
        self.add_cc_boilerplate(cc, state, handle);

        match cc {
            ChanceCard::PropertyTax => self.mod_cc_property_tax(state, handle),
            ChanceCard::Level1Rent => self.mod_cc_level_1_rent(state),
            ChanceCard::AllToParking => self.mod_cc_all_to_parking(state, handle),
            _ => panic!("choiceful cc passed to Game.mod_choiceless_cc()"),
        }
    }

    /// Modify `state` according to the effects of the 'property tax'
    /// chance card. `parent_handle` is the handle of `state`'s parent.
    fn mod_cc_property_tax(&self, state: &mut StateDiff, parent_handle: usize) {
        let mut tax = 0;

        // Tax $50 per property owned
        for (_, prop) in self.diff_owned_properties(parent_handle) {
            if prop.owner == self.diff_current_player(parent_handle) {
                tax += 50;
            }
        }

        // Clone the players
        let mut updated_players = self.diff_players(parent_handle).clone();
        // Update the players based on the calculated tax
        updated_players[self.diff_current_player(parent_handle)].balance -= tax;
        // Set the players diff
        state.set_players_diff(updated_players);
    }

    /// Modify `state` according to the effects of the 'level 1 rent' chance card.
    fn mod_cc_level_1_rent(&self, state: &mut StateDiff) {
        // Set the diff to 2 rounds (player_count * 2 turns per player)
        state.set_level_1_rent_diff(self.agents.len() as u8 * 2);
    }

    /// Modify `state` according to the effects of the 'all to parking'
    /// chance card. `parent_handle` is the handle of `state`'s parent.
    fn mod_cc_all_to_parking(&self, state: &mut StateDiff, parent_handle: usize) {
        // Clone players
        let mut updated_players = self.diff_players(parent_handle).clone();

        // Move every player who's not in jail to free parking
        for player in &mut updated_players {
            if !player.in_jail {
                player.position = JAIL_POSITION;
            }
        }

        // Set the diff
        state.set_players_diff(updated_players);
    }

    /// Modify `state` according to what happens after you get any chance card:
    /// - Set `next_move` to `Roll`
    /// - Update `current_player`
    /// - Update `seen_ccs_head` if needed
    ///
    /// This does not apply the effects of that specific chance card.
    fn add_cc_boilerplate(&self, card: ChanceCard, state: &mut StateDiff, handle: usize) {
        // After you get a chance card, the next move is a roll
        state.next_move = MoveType::Roll;
        // And it's the next player's turn
        state.set_current_player_diff((self.diff_current_player(handle) + 1) % self.agents.len());
        // Update the seen_ccs_head if needed
        if self.diff_seen_ccs(handle).len() == 21 {
            state.set_seen_ccs_head_diff((self.diff_seen_ccs_head(handle) + 1) % 21);
        } else {
            let mut updated_seen_ccs = self.diff_seen_ccs(handle).clone();
            updated_seen_ccs.push(card);
            state.set_seen_ccs_diff(updated_seen_ccs);
        }
    }
}
