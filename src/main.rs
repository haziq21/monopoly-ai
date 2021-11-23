#![allow(dead_code, unused_mut, unused_variables)]

use std::collections::HashSet;

mod helpers;
use helpers::*;

const NUM_PLAYERS: usize = 2;

enum StateType {
    Chance(f64),
    Choice,
}

impl StateType {
    fn probability(&self) -> f64 {
        match self {
            StateType::Chance(p) => *p,
            _ => unreachable!(),
        }
    }
}

#[derive(Copy, Clone)]
struct Player {
    in_jail: bool,
    position: u8,
    balance: u16,
    doubles_rolled: u8,
}

struct State {
    r#type: StateType,
    players: [Player; NUM_PLAYERS],
    current_player_index: usize,
    next_move_is_chance: bool,
}

impl State {
    /*********        HELPER FUNCTIONS        *********/

    /// The player whose turn it currently is.
    fn current_player(&mut self) -> &mut Player {
        &mut self.players[self.current_player_index]
    }

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

    fn roll_to_cc_effects(&mut self) -> Vec<State> {
        // PLayer did not land on a chance card tile so don't do anything
        if !CC_POSITIONS.contains(&self.current_player().position) {
            self.next_move_is_chance = true;
            return vec![];
        }

        let children = vec![];

        // Chance card: -$50 per property owned
        let property_penalty = State {
            r#type: StateType::Chance(self.r#type.probability() / 21),
            ..*self
        };
        let mut property_penalty_has_effect = false;

        // Deduct $50 per property owned
        // for prop in propertyPenalty.props {
        //     if prop.owner == propertyPenalty.board.currentPlayerIndex {
        //         propertyPenalty.currentPlayer.balance -= 50;
        //         propertyPenaltyIsDifferent = true;
        //     }
        // }

        children
    }

    /// Return child nodes of the current game state that can be reached by rolling dice.
    fn roll_effects(&mut self) -> Vec<State> {
        let mut children = vec![];

        // Get the player out of jail if they're in jail
        if self.current_player().in_jail {
            // Try rolling doubles to get out of jail
            let double_probabilities = State::roll_for_doubles(3);

            // Loop through all possible dice results
            for roll in double_probabilities {
                // Derive a new game state from the current game state
                let mut new_state = State {
                    r#type: StateType::Chance(roll.probability),
                    ..*self
                };

                // We didn't manage to roll doubles
                if !roll.is_double {
                    // $100 penalty for not rolling doubles
                    new_state.current_player().balance -= 100;
                }

                // Update the current player's position
                new_state.move_by(roll.sum);

                // chanceEffects(newState);

                // Store the updated state
                children.push(new_state);
            }
        }
        // Otherwise, play as normal
        else {
            // Loop through all possible dice results
            for roll in &*SIGNIFICANT_ROLLS {
                // Derive a new game state from the current game state
                let mut new_state = State {
                    r#type: StateType::Chance(roll.probability),
                    ..*self
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

                // Store the new game state
                children.push(new_state);
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

fn main() {
    let _properties = [
        build_property(1, Color::Brown, 60, [70, 130, 220, 370, 750]),
        build_property(3, Color::Brown, 60, [70, 130, 220, 370, 750]),
        build_property(5, Color::LightBlue, 100, [80, 140, 240, 410, 800]),
        build_property(6, Color::LightBlue, 100, [80, 140, 240, 410, 800]),
        build_property(8, Color::LightBlue, 120, [100, 160, 260, 440, 860]),
        build_property(10, Color::Pink, 140, [110, 180, 290, 460, 900]),
        build_property(12, Color::Pink, 140, [110, 180, 290, 460, 900]),
        build_property(13, Color::Pink, 160, [130, 200, 310, 490, 980]),
        build_property(14, Color::Orange, 180, [140, 210, 330, 520, 1000]),
        build_property(15, Color::Orange, 180, [140, 210, 330, 520, 1000]),
        build_property(17, Color::Orange, 200, [160, 230, 350, 550, 1100]),
        build_property(19, Color::Red, 220, [170, 250, 380, 580, 1160]),
        build_property(21, Color::Red, 220, [170, 250, 380, 580, 1160]),
        build_property(22, Color::Red, 240, [190, 270, 400, 610, 1200]),
        build_property(23, Color::Yellow, 260, [200, 280, 420, 640, 1300]),
        build_property(24, Color::Yellow, 260, [200, 280, 420, 640, 1300]),
        build_property(26, Color::Yellow, 280, [220, 300, 440, 670, 1340]),
        build_property(28, Color::Green, 300, [230, 320, 460, 700, 1400]),
        build_property(30, Color::Green, 300, [230, 320, 460, 700, 1400]),
        build_property(31, Color::Green, 320, [250, 340, 480, 730, 1440]),
        build_property(33, Color::Blue, 350, [270, 360, 510, 740, 1500]),
        build_property(35, Color::Blue, 400, [300, 400, 560, 810, 1600]),
    ];

    let _loc_positions = HashSet::from([7, 16, 25, 34]);

    let _cc_positions = HashSet::from([2, 4, 11, 20, 29, 32]);
    let _prop_positions = HashSet::from([
        1, 3, 5, 6, 8, 10, 12, 13, 14, 15, 17, 19, 21, 22, 23, 24, 26, 28, 30, 31, 33, 35,
    ]);

    println!("{:?}", *SIGNIFICANT_ROLLS);
    println!("{}", *SINGLE_PROBABILITY);
}
