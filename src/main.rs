#![allow(dead_code, unused_mut, unused_variables)]

use std::collections::HashMap;
use std::fmt;
use std::time::Instant;

mod helpers;
use helpers::*;

#[derive(Clone, Debug)]
struct State {
    r#type: StateType,
    players: Vec<Player>,
    property_owners: HashMap<usize, usize>,
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
    /*********        INITIALISATION INTERFACES        *********/

    fn origin(player_count: u8) -> State {
        State {
            r#type: StateType::Choice,
            players: Player::multiple_new(2),
            property_owners: HashMap::new(),
            current_player_index: 0,
            next_move_is_chance: true,
            active_cc: None,
            lvl1rent_cc: 0,
        }
    }

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

    /*********        STATE GENERATION        *********/

    /// Return child nodes of the current game state that can be reached from a location tile.
    fn loc_choice_effects(&self) -> Vec<State> {
        vec![]
    }

    /// Return child nodes of the current game state that can be reached from a property tile.
    fn prop_choice_effects(&self) -> Vec<State> {
        // if self.current_property().unwrap().
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
    fn cc_chance_effects(&mut self) -> Vec<State> {
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
            children.push(State {
                r#type: StateType::Chance(self.r#type.probability() * (amount as f64) / 21.),
                active_cc: Some(card),
                next_move_is_chance: false,
                ..self.clone()
            });
        }

        let total_children_probability = children
            .iter()
            .map(|child| child.r#type.probability())
            .sum::<f64>()
            // Add this probability to account for the "all players to jail"
            // chance card that's implemented in `roll_effects()`
            + self.r#type.probability() / 21.;

        // Correct the current state's probability to
        // account for the chance card child states
        if let StateType::Chance(p) = self.r#type {
            self.r#type = StateType::Chance(p - total_children_probability);
        };

        // total_children_probability != self.r#type.probability() when at
        // least one chance card has no effect. So `self.probability() == 0`
        // when every chance card has an effect.

        // `self` is the state where none of the chance cards apply, so it's the next player's turn.
        self.setup_next_player();

        children
    }

    /// Return child nodes of the current game state that can be reached by rolling dice.
    fn roll_effects(&mut self) -> Vec<State> {
        let mut children = vec![];

        // Probability of rolling (without doubles) to a chance
        // card tile and getting the "all to parking" card
        let mut atp_singles_probability = 0.;
        // Probability of rolling (with doubles) to a chance
        // card tile and getting the "all to parking" card
        let mut atp_doubles_probability = 0.;

        // Performs some other actions before pushing `state` to `children`
        let mut push_state = |mut state: State, rolled_doubles: bool| {
            // Player landed on a chance card tile
            if CC_POSITIONS.contains(&state.current_player().position) {
                let atp_probability = state.r#type.probability() / 21.;

                // Effects of rolling to a chance card tile
                let mut cc_effects = state.cc_chance_effects();
                for s in &mut cc_effects {
                    if s.lvl1rent_cc > 0 {
                        s.lvl1rent_cc -= 1
                    }
                }

                // Chance card: Move all players not in jail to free
                // parking. This is implemented here (instead of in
                // `cc_chance_effects()`) for optimisation purposes.
                if rolled_doubles {
                    atp_doubles_probability += atp_probability;
                } else {
                    atp_singles_probability += atp_probability;
                }

                children.splice(children.len().., cc_effects);

                match state.r#type {
                    // No need to push `state` to children if its probability is 0
                    StateType::Chance(p) if p == 0. => return,
                    _ => (),
                }
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

                push_state(new_state, roll.is_double);
            }
        }

        // Chance card: Move all players not in jail to free parking.
        // This is implemented here (instead of in
        // `cc_chance_effects()`) for optimisation purposes.

        // Set up "all to parking"'s single-roll state
        let mut atp_singles = State {
            r#type: StateType::Chance(atp_singles_probability),
            ..self.clone()
        };
        atp_singles.current_player().doubles_rolled = 0;

        // Set up "all to parking"'s double-roll state
        let mut atp_doubles = State {
            r#type: StateType::Chance(atp_doubles_probability),
            ..self.clone()
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
            children.push(atp);
        }

        // The chance card "move all players not in jail to free parking"
        // may generate identical child states, so we have to merge their probabilities
        // State::merge_probabilities(&mut children, self.current_player_index);

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

    fn static_eval(&self) -> Vec<f64> {
        vec![1.0]
    }

    fn minimax(&self, depth: u64) {}
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
    let start = Instant::now();

    let children = State::origin(2).children();

    let duration = start.elapsed();

    print_states(&children);
    println!("Time elapsed: {:?}", duration);
}
