#![allow(dead_code, unused_mut, unused_variables)]

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
            StateType::Chance(p) => format!("Probability: \x1b[33m{}\x1b[0m", p),
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
                    match self.r#type {
                        StateType::Chance(_) => "chance",
                        StateType::Choice => "choice",
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

        // It's the next player's turn now
        self.setup_next_player();
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

    fn choice_effects(&self) -> Vec<State> {
        let mut children = vec![];

        children
    }

    /// Return child nodes of the current game state that
    /// can be reached by rolling to a chance card tile.
    /// This modifies `self` and is only called in `roll_effects()`.
    fn roll_to_cc_effects(&mut self) -> Vec<State> {
        // PLayer did not land on a chance card tile so don't do anything
        if !CC_POSITIONS.contains(&self.current_player().position) {
            self.next_move_is_chance = false;
            return vec![];
        }

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
        property_penalty.current_player().balance -= property_penalty_deduction;

        // Only add a new child state if it's different
        if property_penalty_deduction > 0 {
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
        let mut push_cc_effect = |mut state: State| {
            let mut cc_effects = state.roll_to_cc_effects();
            for s in &mut cc_effects {
                if s.lvl1rent_cc > 0 {
                    s.lvl1rent_cc -= 1
                }
            }

            children.splice(children.len().., cc_effects);

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

                push_cc_effect(new_state);
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

                push_cc_effect(new_state);
            }
        }

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
        players: build_players(2),
        current_player_index: 0,
        next_move_is_chance: true,
        active_cc: None,
        lvl1rent_cc: 0,
    };

    let children = origin_state.children();
    print_states(&children);
}
