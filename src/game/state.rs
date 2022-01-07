#![allow(dead_code, unused_mut, unused_variables)]

use std::collections::HashMap;
use std::fmt;

use super::globals::*;

#[derive(Copy, Clone, Debug)]
/// One of two types that a game state could be.
pub enum StateType {
    /// A game state that was achieved by chance (i.e. by rolling the dice).
    Chance(f64),
    /// A game state that was achieved by making a choice.
    Choice,
}

impl StateType {
    /// Return the associated value if `self` is
    /// a `Statetype::Chance`, and panic otherwise.
    pub fn probability(&self) -> f64 {
        match self {
            StateType::Chance(p) => *p,
            _ => unreachable!(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
/// Information about a property related to its ownership.
pub struct OwnedProperty {
    /// The index of the player who owns this property
    pub owner: usize,
    /// The rent level of this property.
    /// Rent level starts at 1 and caps out at 5.
    pub rent_level: u8,
}

impl OwnedProperty {
    /// Raise the rent level by one, if possible. Return whether this had any effect.
    pub fn raise_rent(&mut self) -> bool {
        if self.rent_level < 5 {
            self.rent_level += 1;
            true
        } else {
            false
        }
    }

    /// Lower the rent level by one, if possible. Return whether this had any effect.
    pub fn lower_rent(&mut self) -> bool {
        if self.rent_level > 1 {
            self.rent_level -= 1;
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Debug)]
/// The state of a game as a node on the game tree.
pub struct State {
    /// The type of the state - either chance or choice.
    pub r#type: StateType,
    /// The players playing the game
    pub players: Vec<Player>,
    /// A hashmap of properties owned by the players, with the
    /// keys being the position of a property around the board.
    pub owned_properties: HashMap<u8, OwnedProperty>,
    /// The index of the player (from `players`) whose turn it currently is.
    pub current_player_index: usize,
    /// Whether the child states are achievable through chance
    /// (dice rolling) or choice (players making decisions).
    pub next_move_is_chance: bool,
    /// The choiceful chance card that a player needs to act on in child states.
    /// This being `Some(_)` implies that `next_move_is_chance == false`.
    pub active_cc: Option<ChanceCard>,
    /// The number of rounds to go before the effect of the chance card
    /// "all players pay level 1 rent for the next two rounds" wears off.
    pub lvl1rent_cc: u8,
    /// The chance cards that have been used, ordered from least recent to most recent.
    pub seen_ccs: Vec<ChanceCard>,
    /// The child nodes of this state.
    pub children: Vec<Box<State>>,
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

    /// Create a new game state with `player_count` players.
    pub fn new(player_count: usize) -> State {
        State {
            r#type: StateType::Choice,
            players: Player::multiple_new(player_count),
            owned_properties: HashMap::new(),
            current_player_index: 0,
            next_move_is_chance: true,
            active_cc: None,
            lvl1rent_cc: 0,
            seen_ccs: vec![],
            children: vec![],
        }
    }

    /*********        ALIASES (FOR CONVENIECE)        *********/

    /// A mutable reference to the player whose turn it currently is.
    pub fn current_player(&mut self) -> &mut Player {
        &mut self.players[self.current_player_index]
    }

    /// The position of the current player.
    pub fn current_position(&self) -> u8 {
        self.players[self.current_player_index].position
    }

    /// The property the current player is on.
    pub fn current_property(&self) -> Option<&Property> {
        PROPERTIES.get(&self.current_position())
    }

    /// A mutable reference to the owned property the current player is on.
    pub fn current_owned_property(&mut self) -> Option<&mut OwnedProperty> {
        self.owned_properties.get_mut(&self.current_position())
    }

    /// The owned property located at `key`.
    pub fn owned_prop(&mut self, key: &u8) -> Option<&mut OwnedProperty> {
        self.owned_properties.get_mut(key)
    }

    /*********        HELPER FUNCTIONS        *********/

    /// Move the current player by the specified amount of tiles.
    pub fn move_by(&mut self, amount: u8) {
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
    pub fn setup_next_player(&mut self) {
        // This player didn't roll doubles in their previous turn
        if self.current_player().doubles_rolled == 0 {
            // Change the current_player_index to the index of the next player
            self.current_player_index = (self.current_player_index + 1) % NUM_PLAYERS;
        }

        // The player whose turn it is next rolls the dice
        self.next_move_is_chance = true;
    }

    /// Send a player to jail.
    pub fn send_to_jail(&mut self, player_index: usize) {
        // Set the player's position to jail
        self.players[player_index].position = 9;
        self.players[player_index].in_jail = true;

        // Reset the doubles counter
        self.players[player_index].doubles_rolled = 0;
    }

    /// Send the current player to jail.
    pub fn send_current_to_jail(&mut self) {
        self.send_to_jail(self.current_player_index)
    }

    pub fn or_choice_clone(&self, children: Vec<Box<State>>) -> Vec<Box<State>> {
        if children.len() > 0 {
            children
        } else {
            vec![Box::new(self.clone_to_choice())]
        }
    }

    /// Return a clone of `self` with the state type as `StateType::Choice`.
    pub fn clone_to_choice(&self) -> State {
        State {
            r#type: StateType::Choice,
            ..self.clone()
        }
    }

    /// Return every possible result of attempting to roll doubles for a maximum of `tries` times.
    pub fn roll_for_doubles(tries: i32) -> Vec<DiceRoll> {
        /*
         *  Let P(S) be the probability that a double is not attained in one roll.
         *  Let P(r) be the probability of obtaining a specific dice configuration
         *  `r` after one roll. `SIGNIFICANT_ROLLS` demonstrates all possible
         *  "specific dice configurations".
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

    pub fn rollout(&self) -> u8 {
        0
    }
}

pub fn print_states(states: &Vec<Box<State>>) {
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
