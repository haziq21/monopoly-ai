mod globals;
use globals::*;

mod agent;
pub use agent::Agent;

mod state;
use state::{OwnedProperty, State, StateType};

pub struct Game {
    pub agents: Vec<Agent>,
    pub current_state: Box<State>,
}

impl Game {
    /*********        OUTWARD-FACING INTERFACES        *********/

    /// Create a new game.
    pub fn new(agents: Vec<Agent>) -> Game {
        let player_count = agents.len();
        Game {
            agents,
            current_state: Box::new(State::new(player_count)),
        }
    }

    /// Play the game until it ends.
    pub fn play(&mut self) {
        // Placeholder
        // self.minimax(&mut Box::new(State::new(self.player_count())), 2);
    }

    /*********        HELPER FUNCTIONS        *********/

    /// Number of players playing the game.
    fn player_count(&self) -> usize {
        self.agents.len()
    }

    /// Return the index of the player who would most likely "back out" of an auction the last.
    fn auction_winner(&self) -> usize {
        0
    }

    /*********        CHOICEFUL CHANCE CARD EFFECTS        *********/

    pub fn cc_rent_level_to(&self, state: &State, n: u8) -> Vec<Box<State>> {
        let mut children = vec![];

        for (pos, prop) in &state.owned_properties {
            // "RentLvlTo5" only applies to your properties (not opponents)
            if n == 5 && prop.owner != state.current_player_index {
                continue;
            }

            // Don't need to add another child node if the rent level is already at its max/min
            if prop.rent_level != n {
                let mut child = state.clone_to_choice();
                child.owned_properties.get_mut(&pos).unwrap().rent_level = n;
                children.push(Box::new(child));
            }
        }

        state.or_choice_clone(children)
    }

    pub fn cc_rent_change_for_set(&self, state: &State, increase: bool) -> Vec<Box<State>> {
        let mut children = vec![];

        // Loop through all the color sets
        for (_, positions) in PROPS_BY_COLOR.iter() {
            let mut new_state = state.clone_to_choice();
            let mut has_effect = false;

            // Loop through all the properties in this color set
            for pos in positions {
                if let Some(prop) = new_state.owned_properties.get_mut(&pos) {
                    if increase {
                        has_effect |= prop.raise_rent();
                    } else {
                        has_effect |= prop.lower_rent();
                    }
                }
            }

            // Only store the new state if it's different
            if has_effect {
                children.push(Box::new(new_state));
            }
        }

        state.or_choice_clone(children)
    }

    pub fn cc_rent_change_for_side(&self, state: &State, increase: bool) -> Vec<Box<State>> {
        // Possible child states, in clockwise order of affected area.
        // E.g. children[0] is the state where the first side of the board is
        // affected and children[3] is the state where the last side is affected.
        let mut children = vec![Box::new(state.clone_to_choice()); 4];

        // Bitmap of whether the states in `children` are any different from `self`.
        // Rightmost bit indicates whether `children[0]` is different from `self` and
        // fourthmost bit from the right indicates whether `children[3]` is different.
        let mut has_effect: u8 = 0;

        // Loop through the positions of all the owned properties
        for pos in state.owned_properties.keys() {
            // The side of the board `pos` is on - 0 is the first side (with 'go'
            // and 'jail') and 3 is the last side (with 'go to jail' and 'go')
            let i = pos / 9;
            let prop = children[i as usize].owned_properties.get_mut(&pos).unwrap();
            let changed = if increase {
                prop.raise_rent()
            } else {
                prop.lower_rent()
            };

            // Update the bitmap accordingly
            has_effect |= (changed as u8) << i;
        }

        // Remove the states that didn't have an effect
        for c in (0..4).rev() {
            if has_effect & (1 << c) == 0 {
                children.swap_remove(c);
            }
        }

        state.or_choice_clone(children)
    }

    pub fn cc_rent_dec_for_neighbours(&self, state: &State) -> Vec<Box<State>> {
        let mut children = vec![];

        for (pos, prop) in &state.owned_properties {
            // Skip if this property isn't owned by the current player
            if prop.owner != state.current_player_index {
                continue;
            }

            let mut new_state = state.clone_to_choice();
            let mut has_effect = false;

            // Raise this property's rent level
            has_effect |= new_state
                .owned_properties
                .get_mut(&pos)
                .unwrap()
                .raise_rent();

            // Lower neighbours' rent levels (if they're owned)
            for n_pos in PROPERTY_NEIGHBOURS[&pos] {
                if let Some(n_prop) = new_state.owned_properties.get_mut(&n_pos) {
                    has_effect |= n_prop.lower_rent();
                }
            }

            // Store new state if it's different
            if has_effect {
                children.push(Box::new(new_state));
            }
        }

        state.or_choice_clone(children)
    }

    pub fn cc_bonus(&self, state: &State) -> Vec<Box<State>> {
        let mut children = vec![];

        for i in 0..state.players.len() {
            // Skip the current player
            if i == state.current_player_index {
                continue;
            }

            let mut new_state = state.clone_to_choice();

            // Award $200 bonus to this player
            new_state.current_player().balance += 200;

            // Award $200 bonus to an opponent
            new_state.players[i].balance += 200;

            children.push(Box::new(new_state));
        }

        // No need for `self.children_or_clone(children)`
        // because we know there's at least one opponent
        children
    }

    pub fn cc_swap_property(&self, state: &State) -> Vec<Box<State>> {
        let mut children = vec![];
        let mut my_props = vec![];
        let mut opponent_props = vec![];

        // Loop through all owned properties to sort them out by ownership
        for (&pos, prop) in &state.owned_properties {
            if prop.owner == state.current_player_index {
                my_props.push(pos);
            } else {
                opponent_props.push(pos);
            }
        }

        // Loop through all the sorted properties
        for my_pos in my_props {
            for opponent_pos in &opponent_props {
                let mut new_state = state.clone_to_choice();

                // Get the owners of the properties
                let opponent = new_state.owned_properties[&opponent_pos].owner;
                let me = new_state.owned_properties[&my_pos].owner;
                // Swap properties
                new_state.owned_prop(&my_pos).unwrap().owner = opponent;
                new_state.owned_prop(&opponent_pos).unwrap().owner = me;

                children.push(Box::new(new_state));
            }
        }

        // No need for `state.children_or_clone(children)`
        // because we know there's at least one opponent
        children
    }

    pub fn cc_opponent_to_jail(&self, state: &State) -> Vec<Box<State>> {
        let mut children = vec![];

        for i in 0..state.players.len() {
            // Skip the current player
            if i == state.current_player_index {
                continue;
            }

            // Send the opponent to jail
            let mut new_state = state.clone_to_choice();
            new_state.send_to_jail(i);

            children.push(Box::new(new_state));
        }

        // No need for `state.children_or_clone(children)`
        // because we know there's at least one opponent
        children
    }

    pub fn cc_move_to_any_property(&self, state: &State) -> Vec<Box<State>> {
        let mut children = vec![];

        for &pos in PROP_POSITIONS.iter() {
            let mut new_state = state.clone_to_choice();

            // Player can move to any property on the board
            new_state.current_player().position = pos;
            // Effects of landing on the property
            children.splice(children.len().., self.prop_choice_effects(&new_state));
        }

        children
    }

    /*********        STATE GENERATION        *********/

    /// This function requires access to the player weights (for auctioning),
    /// which is why state generation is implemented on `Game` rather than on `State`.
    fn prop_full_effects(&self, state: &State) -> Vec<Box<State>> {
        let current_pos = state.current_position();

        if let Some(prop) = state.owned_properties.get(&current_pos) {
            let mut new_state = state.clone_to_choice();

            // The current player owes rent to the owner of this property
            if prop.owner != new_state.current_player_index {
                let balance_due = if new_state.lvl1rent_cc > 0 {
                    PROPERTIES[&current_pos].rents[0]
                } else {
                    PROPERTIES[&current_pos].rents[prop.rent_level as usize - 1]
                };

                // Pay the owner...
                new_state.players[prop.owner].balance += balance_due;
                // ...using the current player's money
                new_state.current_player().balance -= balance_due;
            }

            // Raise the rent level
            new_state.current_owned_property().unwrap().raise_rent();

            // It's the end of this player's turn
            new_state.setup_next_player();

            vec![Box::new(new_state)]
        } else {
            // Choose to auction this property
            let mut no_buy = state.clone_to_choice();
            let auction_winner = self.auction_winner();
            no_buy.players[auction_winner].balance -= PROPERTIES[&current_pos].price;
            no_buy.owned_properties.insert(
                current_pos,
                OwnedProperty {
                    owner: auction_winner,
                    rent_level: 1,
                },
            );

            // Choose to buy this property
            let mut buy_prop = state.clone_to_choice();
            buy_prop.current_player().balance -= PROPERTIES[&current_pos].price;
            buy_prop.owned_properties.insert(
                current_pos,
                OwnedProperty {
                    owner: buy_prop.current_player_index,
                    rent_level: 1,
                },
            );

            vec![Box::new(no_buy), Box::new(buy_prop)]
        }
    }

    /// Return child nodes of a game state that can be reached from a location tile.
    fn loc_choice_effects(&self, state: &State) -> Vec<Box<State>> {
        let mut children = vec![];

        for &pos in PROP_POSITIONS.iter() {
            let mut new_state = state.clone_to_choice();

            // Play $100
            new_state.current_player().balance -= 100;
            // Player can teleport to any property on the board
            new_state.current_player().position = pos;
            // Effects of landing on the property
            children.splice(children.len().., self.prop_full_effects(&new_state));
        }

        // There's also the option to do nothing
        children.push(Box::new(state.clone_to_choice()));

        children
    }

    /// Return child nodes of a game state that can be reached by buying or auctioning a property
    fn prop_choice_effects(&self, state: &State) -> Vec<Box<State>> {
        let current_pos = state.current_position();

        // Choose to auction this property
        let mut no_buy = state.clone_to_choice();
        let auction_winner = self.auction_winner();
        no_buy.players[auction_winner].balance -= PROPERTIES[&current_pos].price;
        no_buy.owned_properties.insert(
            current_pos,
            OwnedProperty {
                owner: auction_winner,
                rent_level: 1,
            },
        );

        // Choose to buy this property
        let mut buy_prop = state.clone_to_choice();
        buy_prop.current_player().balance -= PROPERTIES[&buy_prop.current_position()].price;
        buy_prop.owned_properties.insert(
            buy_prop.current_position(),
            OwnedProperty {
                owner: buy_prop.current_player_index,
                rent_level: 1,
            },
        );

        vec![Box::new(no_buy), Box::new(buy_prop)]
    }

    /// Return child nodes of the current game state that can be
    /// reached by making a decision from a chance card tile.
    fn cc_choice_effects(&self, state: &State) -> Vec<Box<State>> {
        let mut children = match state.active_cc.unwrap() {
            ChanceCard::RentLvlTo1 => self.cc_rent_level_to(state, 1),
            ChanceCard::RentLvlTo5 => self.cc_rent_level_to(state, 5),
            ChanceCard::RentLvlIncForSet => self.cc_rent_change_for_set(state, true),
            ChanceCard::RentLvlDecForSet => self.cc_rent_change_for_set(state, false),
            ChanceCard::RentLvlIncForBoardSide => self.cc_rent_change_for_side(state, true),
            ChanceCard::RentLvlDecForBoardSide => self.cc_rent_change_for_side(state, false),
            ChanceCard::RentLvlDecForNeighbours => self.cc_rent_dec_for_neighbours(state),
            ChanceCard::BonusForYouAndOpponent => self.cc_bonus(state),
            ChanceCard::SwapProperty => self.cc_swap_property(state),
            ChanceCard::SendOpponentToJail => self.cc_opponent_to_jail(state),
            ChanceCard::MoveToAnyProperty => self.cc_move_to_any_property(state),
        };

        // Reset the active chance card
        for child in &mut children {
            child.active_cc = None;
        }

        children
    }

    /// Return child nodes of the current game state that can be reached by making a choice.
    fn choice_effects(&self, state: &State) -> Vec<Box<State>> {
        // The player landed on a location tile
        let mut children = if LOC_POSITIONS.contains(&state.current_position()) {
            self.loc_choice_effects(state)
        }
        // The player landed on a property tile
        else if PROP_POSITIONS.contains(&state.current_position()) {
            self.prop_choice_effects(state)
        }
        // The player landed on a chance card tile
        else if CC_POSITIONS.contains(&state.current_position()) {
            self.cc_choice_effects(state)
        } else {
            unreachable!();
        };

        for child in &mut children {
            child.setup_next_player();
        }

        children
    }

    /// Return child nodes of the current game state that
    /// can be reached by rolling to a chance card tile.
    /// This modifies `self` and is only called in `roll_effects()`.
    fn cc_chance_effects(&self, state: &mut State) -> Vec<Box<State>> {
        let mut children = vec![];
        let unit_probability = state.r#type.probability() / 21.;

        // Chance card: -$50 per property owned
        let mut property_penalty = State {
            r#type: StateType::Chance(unit_probability),
            ..state.clone()
        };
        let mut property_penalty_deduction = 0;

        // Deduct $50 per property owned
        for (_, prop) in &property_penalty.owned_properties {
            if prop.owner == property_penalty.current_player_index {
                property_penalty_deduction += 50;
            }
        }
        // Only add a new child state if it's different
        if property_penalty_deduction > 0 {
            property_penalty.current_player().balance -= property_penalty_deduction;
            property_penalty.setup_next_player();
            children.push(Box::new(property_penalty));
        }

        // Chance card: Pay level 1 rent for 2 rounds
        children.push(Box::new(State {
            r#type: StateType::Chance(unit_probability),
            lvl1rent_cc: (state.players.len() * 2) as u8 + 1,
            ..state.clone()
        }));

        // The chance card "Move all players not in jail to free parking"
        // is implemented in `roll_effects()` for optimisation purposes.

        // Chance cards that require the player to make a choice
        let choiceful_ccs = [
            (3, ChanceCard::RentLvlTo1),
            (1, ChanceCard::RentLvlTo5),
            (3, ChanceCard::RentLvlIncForSet),
            (1, ChanceCard::RentLvlDecForSet),
            (1, ChanceCard::RentLvlIncForBoardSide),
            (1, ChanceCard::RentLvlDecForBoardSide),
            (2, ChanceCard::RentLvlDecForNeighbours),
            (2, ChanceCard::BonusForYouAndOpponent),
        ];

        // Push the child states for all the choiceful chance cards
        for (amount, card) in choiceful_ccs {
            children.push(Box::new(State {
                r#type: StateType::Chance(unit_probability * amount as f64),
                active_cc: Some(card),
                next_move_is_chance: false,
                ..state.clone()
            }));
        }

        let total_children_probability = children
            .iter()
            .map(|child| child.r#type.probability())
            .sum::<f64>()
            // Add this probability to account for the "all players to jail"
            // chance card that's implemented in `roll_effects()`
            + unit_probability;

        // Correct the current state's probability to
        // account for the chance card child states
        if let StateType::Chance(p) = state.r#type {
            state.r#type = StateType::Chance(p - total_children_probability);
        };

        // total_children_probability != state.r#type.probability() when at
        // least one choiceless chance card has no effect. So `state.probability() == 0`
        // when every chance card has an effect.

        // `state` is the state where none of the chance cards apply, so it's the next player's turn.
        state.setup_next_player();

        children
    }

    fn in_place_prop_chance_effects(&self, state: &mut State) {
        let current_pos = state.current_position();

        if let Some(prop) = state.owned_properties.get(&current_pos) {
            // The current player owes rent to the owner of this property
            if prop.owner != state.current_player_index {
                let balance_due = if state.lvl1rent_cc > 0 {
                    PROPERTIES[&current_pos].rents[0]
                } else {
                    PROPERTIES[&current_pos].rents[prop.rent_level as usize - 1]
                };

                // Pay the owner...
                state.players[prop.owner].balance += balance_due;
                // ...using the current player's money
                state.current_player().balance -= balance_due;
            }

            // Raise the rent level
            state.current_owned_property().unwrap().raise_rent();

            // It's the end of this player's turn
            state.setup_next_player();
        } else {
            // The player has to decide whether to buy or auction
            state.next_move_is_chance = false;
        }
    }

    /// Return child nodes of the current game state that can be reached by rolling dice.
    fn roll_effects(&self, state: &Box<State>) -> Vec<Box<State>> {
        let mut children = vec![];

        // Probability of rolling (without doubles) to a chance
        // card tile and getting the "all to parking" card
        let mut atp_singles_probability = 0.;
        // Probability of rolling (with doubles) to a chance
        // card tile and getting the "all to parking" card
        let mut atp_doubles_probability = 0.;

        // Performs some other actions before pushing `state` to `children`
        let mut push_state = |mut s: State, rolled_doubles: bool| {
            let current_pos = s.current_position();
            // Player landed on a property tile
            if PROP_POSITIONS.contains(&current_pos) {
                self.in_place_prop_chance_effects(&mut s);
            }
            // Player landed on a chance card tile
            else if CC_POSITIONS.contains(&current_pos) {
                // This line goes above `state.cc_chance_effects()`
                // since that modifies state.r#type.probability()
                let atp_probability = s.r#type.probability() / 21.;

                // Effects of rolling to a chance card tile
                let mut chance_effects = self.cc_chance_effects(&mut s);
                for state in &mut chance_effects {
                    if state.lvl1rent_cc > 0 {
                        state.lvl1rent_cc -= 1
                    }
                }

                children.splice(children.len().., chance_effects);

                // Chance card: Move all players not in jail to free
                // parking. This is implemented here (instead of in
                // `cc_chance_effects()`) for optimisation purposes.
                if rolled_doubles {
                    atp_doubles_probability += atp_probability;
                } else {
                    atp_singles_probability += atp_probability;
                }

                match s.r#type {
                    // No need to push `state` to children if its probability is 0
                    StateType::Chance(p) if p == 0. => return,
                    _ => (),
                }
            } else if CORNER_POSITIONS.contains(&current_pos) {
                // This tile does nothing so it's the next player's turn
                s.setup_next_player();
            } else {
                // The player has to do something according to the tile they're on
                s.next_move_is_chance = false;
            }

            // The previous round has passed
            if s.lvl1rent_cc > 0 {
                s.lvl1rent_cc -= 1
            }

            // Store the new game state
            children.push(Box::new(s));
        };

        // Get the player out of jail if they're in jail
        if state.players[state.current_player_index].in_jail {
            // Try rolling doubles to get out of jail
            let double_probabilities = State::roll_for_doubles(3);

            // Loop through all possible dice results
            for roll in double_probabilities {
                // Derive a new game state from the current game state
                let mut new_state = State {
                    r#type: StateType::Chance(roll.probability),
                    ..*state.clone()
                };

                // We didn't manage to roll doubles
                if !roll.is_double {
                    // $100 penalty for not rolling doubles
                    new_state.current_player().balance -= 100;
                }

                // Update the current player's position
                new_state.move_by(roll.sum);

                // `false` because rolling doubles to get out of
                // jail doesn't count towards your consecutive doubles
                push_state(new_state, false);
            }
        }
        // Otherwise, play as normal
        else {
            // Loop through all possible dice results
            for roll in &*SIGNIFICANT_ROLLS {
                // Derive a new game state from the current game state
                let mut new_state = State {
                    r#type: StateType::Chance(roll.probability),
                    ..*state.clone()
                };

                // Update the current player's position
                new_state.move_by(roll.sum);

                // Check if the player landed on 'go to jail'
                if new_state.current_player().position == 27 {
                    new_state.send_current_to_jail();
                }
                // Check if this roll got doubles
                else if roll.is_double {
                    // Increment the doubles_rolled counter
                    new_state.current_player().doubles_rolled += 1;

                    // Go to jail after three consecutive doubles
                    if new_state.current_player().doubles_rolled == 3 {
                        new_state.send_current_to_jail();
                    }
                } else {
                    // Reset the doubles counter
                    new_state.current_player().doubles_rolled = 0;
                }

                push_state(new_state, roll.is_double);
            }
        }

        // Chance card: Move all players not in jail to free parking.
        // This is implemented here (instead of in `cc_chance_effects()`)
        // for optimisation purposes.

        // Set up "all to parking"'s single-roll state
        let mut atp_singles = State {
            r#type: StateType::Chance(atp_singles_probability),
            ..*state.clone()
        };
        atp_singles.current_player().doubles_rolled = 0;

        // Set up "all to parking"'s double-roll state
        let mut atp_doubles = State {
            r#type: StateType::Chance(atp_doubles_probability),
            ..*state.clone()
        };
        // Note: we know this won't increment to 3 because that logic is already implemented above
        atp_doubles.current_player().doubles_rolled += 1;

        // Move atp players to 'free parking' and update their `lvl1rent_cc`
        for mut atp in [atp_singles, atp_doubles] {
            for player in &mut atp.players {
                if !player.in_jail {
                    player.position = 18;
                }
            }

            if atp.lvl1rent_cc > 0 {
                atp.lvl1rent_cc -= 1
            }

            atp.setup_next_player();
            children.push(Box::new(atp));
        }

        children
    }

    /// Calculates and stores `state.children` (child nodes of `state` on the game tree).
    pub fn find_children(&self, state: &mut Box<State>) {
        if state.children.len() == 0 {
            let children = if state.next_move_is_chance {
                self.roll_effects(state)
            } else {
                self.choice_effects(state)
            };

            state.children = children;
        }
    }

    /*********        MONTE-CARLO TREE SEARCH        *********/

    // TODO
}
