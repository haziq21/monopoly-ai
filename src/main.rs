#![allow(dead_code, unused_mut, unused_variables)]

use bigint::uint::U256;
use std::collections::HashMap;
use std::fmt;

mod helpers;
use helpers::*;

#[derive(Clone, Debug)]
struct State {
    r#type: StateType,
    players: Vec<Player>,
    current_player_index: usize,
    next_move_is_chance: bool,
    active_cc: Option<ChanceCard>,
    lvl1rent_cc: usize,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut metadata = match self.r#type {
            StateType::Chance(p) => format!("Probability: \x1b[33m{:.2}%\x1b[0m", p * 100.),
            StateType::Choice => String::from("No probability"),
        };

        match self.active_cc {
            Some(cc) => metadata.push_str(&format!("\nActive CC: {:?}", cc)),
            None => (),
        }

        if self.lvl1rent_cc > 0 {
            metadata += &format!("\nLvl1rent: {} turns to go", self.lvl1rent_cc);
        }

        let mut players_str = "".to_owned();
        for i in 0..self.players.len() {
            players_str += &format!("{}", self.players[i]);

            if self.current_player_index == i {
                players_str += &format!(
                    " < next: \x1b[36m{}\x1b[0m",
                    if self.next_move_is_chance {
                        "chance"
                    } else {
                        "choice"
                    }
                )
            }

            players_str += "\n";
        }

        write!(f, "{}\n{}", metadata, players_str)
    }
}

impl State {
    /*********        ALIASES (FOR CONVENIECE)        *********/

    /// The player whose turn it currently is.
    fn current_player(&mut self) -> &mut Player {
        &mut self.players[self.current_player_index]
    }

    /// The property the current player is on.
    fn current_property(&mut self) -> Option<&Property> {
        PROPERTIES
            .iter()
            .find(|&prop| prop.position == self.current_player().position)
    }

    /*********        HELPER FUNCTIONS        *********/

    /// Move the current player by the specified amount of tiles.
    fn move_by(&mut self, amount: u8) {
        let new_pos = (self.current_player().position + amount) % 36;

        // Set the player's `in_jail` flag to false if appropriate
        if self.current_player().in_jail && amount != 0 {
            self.current_player().in_jail = false;
        }

        // Give the player $200 if they pass 'Go'
        if new_pos < self.current_player().position {
            self.current_player().balance += 200;
        }

        // Update the position
        self.current_player().position = new_pos;
    }

    /// Change the `current_player_index` to the index of the player
    /// whose turn it is next, but only if the current player didn't
    /// roll doubles (in which case it would be their turn again).
    fn setup_next_player(&mut self) {
        // This player didn't roll doubles in their previous turn
        if self.current_player().doubles_rolled == 0 {
            // Change the current_player_index to the index of the next player
            self.current_player_index = (self.current_player_index + 1) % NUM_PLAYERS;
        }

        // The player whose turn it is next rolls the dice
        self.next_move_is_chance = true;
    }

    /// Send a player to jail.
    fn send_to_jail(&mut self, player_index: usize) {
        // Set the player's position to jail
        self.players[player_index].position = 9;
        self.players[player_index].in_jail = true;

        // Reset the doubles counter
        self.players[player_index].doubles_rolled = 0;
    }

    /// Send the current player to jail.
    fn send_current_to_jail(&mut self) {
        self.send_to_jail(self.current_player_index)
    }

    /// Return every possible result of attempting to roll doubles for a maximum of `tries` times.
    fn roll_for_doubles(tries: i32) -> Vec<DiceRoll> {
        /*
         *  Let P(S) be the probability that a double is not attained in one roll.
         *  Let P(r) be the probability of obtaining a specific dice configuration
         *  `r` after one roll. The return value of `SIGNIFICANT_ROLLS` demonstrates
         *  all possible "specific dice configurations".
         *
         *  When rolling the dice for maximum of `n` times, or stopping
         *  when we get doubles, the probabilities work out as follows:
         *
         *  The probability of the final roll `r` being any double `d` (where the sum
         *  of the dice is `2d`) is given by `sum_(i=0)^(n-1) P(r) * P(S)^i`.
         *
         *  The probability of all `n` rolls being non-doubles (and hence the
         *  final roll being a non-double `r`) is given by `P(r) * P(S)^(n - 1)`.
         *
         *  The following code implements this.
         */
        SIGNIFICANT_ROLLS
            .iter()
            .map(|&roll| {
                DiceRoll {
                    probability: if roll.is_double {
                        let mut double_probability = 0.;

                        // sum_(i=0)^(n-1) P(r) * P(S)^i
                        for i in 0..tries {
                            double_probability += roll.probability * SINGLE_PROBABILITY.powi(i);
                        }

                        double_probability
                    } else {
                        // P(r) * P(S)^(n - 1)
                        roll.probability * SINGLE_PROBABILITY.powi(tries - 1)
                    },
                    ..roll
                }
            })
            .collect()
    }

    /// Return a hash of the current state, used to identify duplicate states.
    ///
    /// The layout of the returned `U256`, from right to left, is as follows:
    /// 1. current player's `in_jail` (1 bit)
    /// 2. current player's `position` (6 bits)
    /// 3. current player's `balance` (16 bits)
    /// 4. current player's `doubles_rolled` (2 bits)
    /// 5. `active_cc` (4 bits)
    /// 6. `lvl1rent_cc` (4 bits)   
    /// 7. Properties and their ownership (132 bits)
    fn hash(&self, active_player_index: usize) -> U256 {
        let curr_player = &self.players[active_player_index];

        // U256::from(struct_property) << horizontal_offset_from_right
        let in_jail = U256::from(curr_player.in_jail as u8) << 0;
        let position = U256::from(curr_player.position) << 1;
        let balance = U256::from(curr_player.balance) << 7;
        let doubles_rolled = U256::from(curr_player.doubles_rolled) << 23;
        let active_cc = match self.active_cc {
            Some(cc) => U256::from(cc as u8),
            None => U256::from(0),
        } << 25;
        let lvl1rent_cc = U256::from(self.lvl1rent_cc) << 29;

        // property22_owner (3 bits), property22_rent_level (3 bits), property21_owner, ...
        let mut props = U256::from(0);
        for (player_i, player) in self.players.iter().enumerate() {
            for (prop_i, rent_level) in &player.property_rents {
                let rent_level_bits = U256::from(*rent_level) << (prop_i * 6);
                let owner_bits = U256::from(player_i) << (prop_i * 6 + 3);

                props = props | rent_level_bits | owner_bits;
            }
        }

        in_jail | position | balance | doubles_rolled | active_cc | lvl1rent_cc | (props << 33)
    }

    /// Merge the probabilities of duplicate states.
    // Note: I'm quite sure the only duplicate state that can be generated is when
    // a player rolls to a chance card tile and gets the "all players to free parking"
    // card, so this function could probably be reduced to just focus on that. TODO I guess.
    fn merge_probabilities(states: &mut Vec<State>, current_player_index: usize) {
        let hashes: Vec<U256> = states
            .iter()
            .map(|s| s.hash(current_player_index))
            .collect();
        let mut seen: HashMap<U256, Vec<usize>> = HashMap::new();

        // Sort every state index by their identity hash
        for (hash, index) in hashes.iter().zip(0..states.len()) {
            if let Some(seen_states) = seen.get_mut(hash) {
                seen_states.push(index);
            } else {
                seen.insert(*hash, vec![index]);
            }
        }

        // probabilities: Vec<(state_index, total_probability)>
        let mut probabilities: Vec<(usize, f64)> = vec![];
        let mut to_remove: Vec<usize> = vec![];

        // Find the indexes of the duplicates
        for indexes in seen.values_mut() {
            if indexes.len() == 1 {
                continue;
            }

            // Calculate the total probability
            let total_probability: f64 = indexes
                .iter()
                .map(|&i| states[i].r#type.probability())
                .sum();

            let extra = &indexes[1..indexes.len()];
            probabilities.push((indexes[0], total_probability));
            to_remove.splice(to_remove.len().., extra.iter().cloned());
        }

        // Merge the duplicate probabilities
        for (i, p) in probabilities {
            states[i].r#type = StateType::Chance(p);
        }

        to_remove.sort_unstable();
        to_remove.reverse();

        // Remove the duplicates
        for i in to_remove {
            states.swap_remove(i);
        }
    }

    /*********        STATE GENERATION        *********/

    /// Return child nodes of the current game state that can be reached from a location tile.
    fn loc_choice_effects(&self) -> Vec<State> {
        vec![]
    }

    /// Return child nodes of the current game state that can be reached from a property tile.
    fn prop_choice_effects(&self) -> Vec<State> {
        vec![]
    }

    /// Return child nodes of the current game state that can be
    /// reached by making a decision from a chance card tile.
    fn cc_choice_effects(&self) -> Vec<State> {
        vec![]
    }

    /// Return child nodes of the current game state that can be reached by making a choice.
    fn choice_effects(&mut self) -> Vec<State> {
        // The player landed on a location tile
        if LOC_POSITIONS.contains(&self.current_player().position) {
            self.loc_choice_effects()
        }
        // The player landed on a property tile
        else if PROP_POSITIONS.contains(&self.current_player().position) {
            self.prop_choice_effects()
        }
        // The player landed on a chance card tile
        else if CC_POSITIONS.contains(&self.current_player().position) {
            self.cc_choice_effects()
        } else {
            unreachable!();
        }
    }

    /// Return child nodes of the current game state that
    /// can be reached by rolling to a chance card tile.
    /// This modifies `self` and is only called in `roll_effects()`.
    fn roll_to_cc_effects(&mut self) -> Vec<State> {
        let mut children = vec![];

        // Chance card: -$50 per property owned
        let mut property_penalty = State {
            r#type: StateType::Chance(self.r#type.probability() / 21.),
            ..self.clone()
        };
        let mut property_penalty_deduction = 0;

        // Deduct $50 per property owned
        for _ in &property_penalty.current_player().property_rents {
            property_penalty_deduction += 50;
        }
        // Only add a new child state if it's different
        if property_penalty_deduction > 0 {
            property_penalty.current_player().balance -= property_penalty_deduction;
            property_penalty.setup_next_player();
            children.push(property_penalty);
        }

        // Chance card: Pay level 1 rent for 2 rounds
        children.push(State {
            r#type: StateType::Chance(self.r#type.probability() / 21.),
            lvl1rent_cc: self.players.len() * 2 + 1,
            ..self.clone()
        });

        // Chance card: Move all players not in jail to free parking
        let mut all_to_parking = State {
            r#type: StateType::Chance(self.r#type.probability() / 21.),
            ..self.clone()
        };
        let mut all_to_parking_has_effect = false;

        // Move players to 'free parking'
        for player in &mut all_to_parking.players {
            if !player.in_jail {
                player.position = 18;
                all_to_parking_has_effect = true;
            }
        }

        // Only add a new child state if it's different
        if all_to_parking_has_effect {
            all_to_parking.setup_next_player();
            children.push(all_to_parking);
        }

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
            children.push(State {
                r#type: StateType::Chance(self.r#type.probability() * (amount as f64) / 21.),
                active_cc: Some(card),
                next_move_is_chance: false,
                ..self.clone()
            });
        }

        let total_children_probability: f64 = children
            .iter()
            .map(|child| child.r#type.probability())
            .sum();

        // Correct the current state's probability to
        // account for the chance card child states
        self.r#type = match self.r#type {
            StateType::Chance(p) => StateType::Chance(p - total_children_probability),
            _ => unreachable!(),
        };

        children
    }

    /// Return child nodes of the current game state that can be reached by rolling dice.
    fn roll_effects(&mut self) -> Vec<State> {
        let mut children = vec![];

        // Performs some other actions before pushing `state` to `children`
        let mut push_state = |mut state: State| {
            // PLayer landed on a chance card tile
            if CC_POSITIONS.contains(&state.current_player().position) {
                // Effects of rolling to a chance card tile
                let mut cc_effects = state.roll_to_cc_effects();
                for s in &mut cc_effects {
                    if s.lvl1rent_cc > 0 {
                        s.lvl1rent_cc -= 1
                    }
                }

                children.splice(children.len().., cc_effects);

                return; // to avoid pushing `state` to children
            } else if CORNER_POSITIONS.contains(&state.current_player().position) {
                // This tile does nothing so it's the next player's turn
                state.setup_next_player();
            } else {
                // The player has to do something according to the tile they're on
                state.next_move_is_chance = false;
            }

            if state.lvl1rent_cc > 0 {
                state.lvl1rent_cc -= 1
            }

            // Store the new game state
            children.push(state);
        };

        // Get the player out of jail if they're in jail
        if self.current_player().in_jail {
            // Try rolling doubles to get out of jail
            let double_probabilities = State::roll_for_doubles(3);

            // Loop through all possible dice results
            for roll in double_probabilities {
                // Derive a new game state from the current game state
                let mut new_state = State {
                    r#type: StateType::Chance(roll.probability),
                    ..self.clone()
                };

                // We didn't manage to roll doubles
                if !roll.is_double {
                    // $100 penalty for not rolling doubles
                    new_state.current_player().balance -= 100;
                }

                // Update the current player's position
                new_state.move_by(roll.sum);

                push_state(new_state);
            }
        }
        // Otherwise, play as normal
        else {
            // Loop through all possible dice results
            for roll in &*SIGNIFICANT_ROLLS {
                // Derive a new game state from the current game state
                let mut new_state = State {
                    r#type: StateType::Chance(roll.probability),
                    ..self.clone()
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

                push_state(new_state);
            }
        }

        // The chance card "move all players not in jail to free parking"
        // may generate identical child states, so we have to merge their probabilities
        State::merge_probabilities(&mut children, self.current_player_index);

        children
    }

    /// Return child nodes of the current game state on the game tree.
    fn children(&mut self) -> Vec<State> {
        if self.next_move_is_chance {
            self.roll_effects()
        } else {
            self.choice_effects()
        }
    }

    /*********        MINIMAX        *********/

    // TODO
}

fn print_states(states: &Vec<State>) {
    for child in states {
        println!("{}", child);
    }

    println!("{} total child states", states.len());

    let total_probability: f64 = states
        .iter()
        .map(|s| match s.r#type {
            StateType::Chance(p) => p,
            StateType::Choice => 0.,
        })
        .sum();

    if total_probability != 0. {
        println!("Total probability: {}", total_probability);
    } else {
        println!("No total probability")
    }
}

fn main() {
    let mut origin_state = State {
        r#type: StateType::Choice,
        players: Player::multiple_new(2),
        current_player_index: 0,
        next_move_is_chance: true,
        active_cc: None,
        lvl1rent_cc: 0,
    };

    let children = origin_state.children();
    print_states(&children);
}
