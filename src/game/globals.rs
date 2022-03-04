use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};
use std::fmt;

#[derive(Debug, Copy, Clone)]
/// A possible outcome of rolling the dice.
pub struct DiceRoll {
    /// The probability of rolling this specific dice configuration.
    pub probability: f64,
    /// The sum of the two dice.
    pub sum: u8,
    /// Whether both the dice resulted in the same number.
    pub is_double: bool,
}

#[derive(PartialEq, Eq, Hash)]
/// The color sets of properties.
pub enum Color {
    Brown,
    LightBlue,
    Pink,
    Orange,
    Red,
    Yellow,
    Green,
    Blue,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
/// Chance cards that require the player to make a choice.
///
/// Note that any chance card that affects a property requires the
/// property to be owned. In the event that such a chance card is
/// received but no one owns a property yet, do nothing.
pub enum ChanceCard {
    /// Set any property's rent level to 1.
    RentTo1,
    /// Set one of your properties' rent level to 5.
    RentTo5,
    /// Choose any color set of which you own a property in, and raise
    /// the rent level of all the properties in that color set by 1.
    SetRentInc,
    /// Choose any color set of which you own a property in, and lower
    /// the rent level of all the properties in that color set by 1.
    SetRentDec,
    /// Choose a side of the board of which you own a property on, and
    /// raise the rent level of all the properties on that side by 1.
    SideRentInc,
    /// Choose a side of the board of which you own a property on, and
    /// lower the rent level of all the properties on that side by 1.
    SideRentDec,
    /// Raise the rent level of any property you own by 1, and lower the
    /// rent levels of that property's neighbors by 1. Note that "neighbours"
    /// refers to the closest property towards the left and right of the
    /// chosen property, regardless of ownership or distance away.
    RentSpike,
    /// You and any opponent you choose recieve $200 from the bank.
    Bonus,
    /// Exchange the ownership of one of your properties
    /// with one of your opponents' properties.
    SwapProperty,
    /// Choose any opponent to send to jail.
    OpponentToJail,
    /// Move to any property tile around the board and
    /// buy, auction, or raise its rent level by 1.
    GoToAnyProperty,
    /// Pay $50 to the bank for every property you own.
    PropertyTax,
    /// All players pay level 1 rent for two rounds.
    Level1Rent,
    /// Move all players who are not in jail to free parking.
    AllToParking,
}

impl ChanceCard {
    pub fn unseen_counts(seen_cards: &Vec<ChanceCard>) -> HashMap<ChanceCard, u8> {
        let mut counts = HashMap::from([
            (ChanceCard::RentTo1, 3),
            (ChanceCard::RentTo5, 1),
            (ChanceCard::SetRentInc, 3),
            (ChanceCard::SetRentDec, 1),
            (ChanceCard::SideRentInc, 1),
            (ChanceCard::SideRentDec, 1),
            (ChanceCard::RentSpike, 2),
            (ChanceCard::Bonus, 2),
            (ChanceCard::SwapProperty, 2),
            (ChanceCard::OpponentToJail, 1),
            (ChanceCard::GoToAnyProperty, 1),
            (ChanceCard::PropertyTax, 1),
            (ChanceCard::Level1Rent, 1),
            (ChanceCard::AllToParking, 1),
        ]);

        for card in seen_cards {
            *counts.get_mut(card).unwrap() -= 1;
        }

        counts
    }

    pub fn is_choiceless(&self) -> bool {
        match self {
            ChanceCard::PropertyTax | ChanceCard::Level1Rent | ChanceCard::AllToParking => true,
            _ => false,
        }
    }
}

/// A property tile on the board.
pub struct Property {
    /// The color set that the property belongs to.
    pub color: Color,
    /// The price of the property.
    pub price: u16,
    /// The rent amount for each rent level of the property.
    /// `rents[0]` would be the rent amount for rent level 1,
    /// and `rents[4]` would be that of rent level 5.
    pub rents: [u16; 5],
}

impl Property {
    /// Creates a new property.
    pub fn new(color: Color, price: u16, rents: [u16; 5]) -> Property {
        Property {
            color,
            price,
            rents,
        }
    }
}

#[derive(Clone, Debug)]
/// A player playing the game.
pub struct Player {
    /// Whether the player is currently in jail.
    pub in_jail: bool,
    /// The player's position around the board. 'Go' is at 0
    /// and 'Mayfair' (the last tile going clockwise) is at 35.
    pub position: u8,
    /// The amount of money the player has.
    pub balance: i32,
    /// The number of consecutive doubles the player has rolled.
    pub doubles_rolled: u8,
}

impl Player {
    /// Return a new player.
    pub fn new() -> Player {
        Player {
            in_jail: false,
            position: 0,
            balance: 1500,
            doubles_rolled: 0,
        }
    }

    /// Move the player on the board.
    pub fn move_by(&mut self, distance: u8) {
        let new_pos = (self.position + distance) % 36;

        // Set the player's `in_jail` flag to false if appropriate
        if self.in_jail && distance != 0 {
            self.in_jail = false;
        }

        // Give the player $200 if they pass 'Go'
        if new_pos < self.position {
            self.balance += 200;
        }

        // Update the position
        self.position = new_pos;
    }

    /// Send the player to jail.
    pub fn send_to_jail(&mut self) {
        // Set the player's position to jail
        self.position = 9;
        self.in_jail = true;

        // Reset the doubles counter
        self.doubles_rolled = 0;
    }
}

impl std::fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pos_color = if self.in_jail { "\x1b[31m" } else { "\x1b[36m" };

        write!(
            f,
            "[{}{:02}\x1b[0m] \x1b[33m{}\x1b[0mdbls \x1b[32m${}\x1b[0m",
            pos_color, self.position, self.doubles_rolled, self.balance
        )
    }
}

pub const DIFF_ID_BRANCH_TYPE: u8 = 7;
pub const DIFF_ID_PLAYERS: u8 = 6;
pub const DIFF_ID_CURRENT_PLAYER: u8 = 5;
pub const DIFF_ID_OWNED_PROPERTIES: u8 = 4;
pub const DIFF_ID_SEEN_CCS: u8 = 3;
pub const DIFF_ID_SEEN_CCS_HEAD: u8 = 2;
pub const DIFF_ID_LEVEL_1_RENT: u8 = 1;

pub const JAIL_POSITION: u8 = 9;
pub const GO_TO_JAIL_POSITION: u8 = 27;

lazy_static! {
    /// Positions of the chance card tiles on the game board.
    pub static ref CC_POSITIONS: HashSet<u8> = HashSet::from([2, 4, 11, 20, 29, 32]);

    /// Positions of the location tiles on the game board.
    pub static ref LOC_POSITIONS: HashSet<u8> = HashSet::from([7, 16, 25, 34]);

    /// Positions of the property tiles on the game board.
    pub static ref PROP_POSITIONS: HashSet<u8> = HashSet::from([
        1, 3, 5, 6, 8, 10, 12, 13, 14, 15, 17, 19, 21, 22, 23, 24, 26, 28, 30, 31, 33, 35,
    ]);

    /// Positions of the corners of the game board.
    pub static ref CORNER_POSITIONS: HashSet<u8> = HashSet::from([0, 9, 18, 27]);

    /// All the properties on the game board, in the form `HashMap<property_position, property>`.
    pub static ref PROPERTIES: HashMap<u8, Property> = HashMap::from([
        (1, Property::new(Color::Brown, 60, [70, 130, 220, 370, 750])),
        (3, Property::new(Color::Brown, 60, [70, 130, 220, 370, 750])),
        (5, Property::new(Color::LightBlue, 100, [80, 140, 240, 410, 800])),
        (6, Property::new(Color::LightBlue, 100, [80, 140, 240, 410, 800])),
        (8, Property::new(Color::LightBlue, 120, [100, 160, 260, 440, 860])),
        (10, Property::new(Color::Pink, 140, [110, 180, 290, 460, 900])),
        (12, Property::new(Color::Pink, 140, [110, 180, 290, 460, 900])),
        (13, Property::new(Color::Pink, 160, [130, 200, 310, 490, 980])),
        (14, Property::new(Color::Orange, 180, [140, 210, 330, 520, 1000])),
        (15, Property::new(Color::Orange, 180, [140, 210, 330, 520, 1000])),
        (17, Property::new(Color::Orange, 200, [160, 230, 350, 550, 1100])),
        (19, Property::new(Color::Red, 220, [170, 250, 380, 580, 1160])),
        (21, Property::new(Color::Red, 220, [170, 250, 380, 580, 1160])),
        (22, Property::new(Color::Red, 240, [190, 270, 400, 610, 1200])),
        (23, Property::new(Color::Yellow, 260, [200, 280, 420, 640, 1300])),
        (24, Property::new(Color::Yellow, 260, [200, 280, 420, 640, 1300])),
        (26, Property::new(Color::Yellow, 280, [220, 300, 440, 670, 1340])),
        (28, Property::new(Color::Green, 300, [230, 320, 460, 700, 1400])),
        (30, Property::new(Color::Green, 300, [230, 320, 460, 700, 1400])),
        (31, Property::new(Color::Green, 320, [250, 340, 480, 730, 1440])),
        (33, Property::new(Color::Blue, 350, [270, 360, 510, 740, 1500])),
        (35, Property::new(Color::Blue, 400, [300, 400, 560, 810, 1600])),
    ]);

    /// Positions of the properties on the game board, sorted by their color set.
    pub static ref PROPS_BY_COLOR: HashMap<Color,Vec<u8>> = HashMap::from([
        (Color::Brown, vec![1, 3]),
        (Color::LightBlue, vec![5, 6, 8]),
        (Color::Pink, vec![10, 12, 13]),
        (Color::Orange, vec![14, 15, 17]),
        (Color::Red, vec![19, 21, 22]),
        (Color::Yellow, vec![23, 24, 26]),
        (Color::Green, vec![28, 30, 31]),
        (Color::Blue, vec![33, 35]),
    ]);

    /// Positions of the properties on the game board, sorted by the side of the board they're on.
    pub static ref PROPS_BY_SIDE: [Vec<u8>; 4] = [
        vec![1, 3, 5, 6, 8],
        vec![10, 12, 13, 14, 15, 17],
        vec![19, 21, 22, 23, 24, 26],
        vec![28, 30, 31, 33, 35]
    ];

    /// Neighbours of properties in the form
    /// `HashMap<prop_pos, [anti_clockwise_neighbour_pos, clockwise_neighbour_pos]>`.
    pub static ref PROPERTY_NEIGHBOURS: HashMap<u8, [u8; 2]> = HashMap::from([
        (1, [35, 3]),
        (3, [1, 5]),
        (5, [3, 6]),
        (6, [5, 8]),
        (8, [6, 10]),
        (10, [8, 12]),
        (12, [10, 13]),
        (13, [12, 14]),
        (14, [13, 15]),
        (15, [14, 17]),
        (17, [15, 19]),
        (19, [17, 21]),
        (21, [19, 22]),
        (22, [21, 23]),
        (23, [22, 24]),
        (24, [23, 26]),
        (26, [24, 28]),
        (28, [26, 30]),
        (30, [28, 31]),
        (31, [30, 33]),
        (33, [31, 35]),
        (35, [33, 1])
    ]);

    /// A vector of all possible dice rolls.
    pub static ref SIGNIFICANT_ROLLS: Vec<DiceRoll> = {
        let mut sig_rolls = vec![];
        let probability = 1. / 36.;

        // Loop through all possible dice results
        for d1 in 1..7 {
            for d2 in 1..7 {
                let sum = d1 + d2;

                // Check if this roll was a double
                if d1 == d2 {
                    // There's only one way to get a double, so push this one to sig_rolls
                    sig_rolls.push(DiceRoll {
                        probability,
                        sum,
                        is_double: true,
                    })
                } else {
                    match sig_rolls.iter().position(|r| r.sum == sum) {
                        // If a roll with the same sum already exists, merge their probabilities
                        Some(i) => sig_rolls[i].probability += probability,
                        // This is a new roll
                        None => sig_rolls.push(DiceRoll {
                            probability,
                            sum,
                            is_double: false,
                        }),
                    }
                }
            }
        }

        sig_rolls
    };

    /// The probability of not rolling a double in one try.
    pub static ref SINGLE_PROBABILITY: f64 = SIGNIFICANT_ROLLS
        .iter()
        .filter(|&r| !r.is_double)
        .map(|&r| r.probability)
        .sum::<f64>();
}

/// Return every significant result of attempting to roll doubles for a maximum of `tries` times.
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
