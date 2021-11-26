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

#[derive(Copy, Clone, Debug)]
/// Chance cards that require the player to make a choice.
///
/// Note that any chance card that affects a property requires the
/// property to be owned. In the event that such a chance card is
/// received but no one owns a property yet, do nothing.
pub enum ChanceCard {
    /// Set any property's rent level to 1.
    RentLvlTo1 = 1,
    /// Set one of your properties' rent level to 5.
    RentLvlTo5 = 2,
    /// Choose any color set of which you own a property in, and raise
    /// the rent level of all the properties in that color set by 1.
    RentLvlIncForSet = 3,
    /// Choose any color set of which you own a property in, and lower
    /// the rent level of all the properties in that color set by 1.
    RentLvlDecForSet = 4,
    /// Choose a side of the board of which you own a property on, and
    /// raise the rent level of all the properties on that side by 1.
    RentLvlIncForBoardSide = 5,
    /// Choose a side of the board of which you own a property on, and
    /// lower the rent level of all the properties on that side by 1.
    RentLvlDecForBoardSide = 6,
    /// Raise the rent level of any property you own by 1, and lower the
    /// rent levels of that property's neighbors by 1. Note that "neighbours"
    /// refers to the closest property towards the left and right of the
    /// chosen property, regardless of ownership or distance away.
    RentLvlDecForNeighbours = 7,
    /// You and any opponent you choose recieve $200 from the bank.
    BonusForYouAndOpponent = 8,
    /// Exchange the ownership of one of your properties
    /// with one of your opponents' properties.
    SwapProperty = 9,
    /// Choose any opponent to send to jail.
    SendOpponentToJail = 10,
    /// Move to any property tile around the board and
    /// buy, auction, or raise its rent level by 1.
    MoveToAnyProperty = 11,
}

/// A property tile on the board.
pub struct Property {
    /// The property's position around the board. 'Go' is at 0
    /// and 'Mayfair' (the last tile going clockwise) is at 35.
    pub position: u8,
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
    fn new(position: u8, color: Color, price: u16, rents: [u16; 5]) -> Property {
        Property {
            position,
            color,
            price,
            rents,
        }
    }
}

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

#[derive(Clone, Debug)]
/// A player playing the game.
pub struct Player {
    /// Whether the player is currently in jail.
    pub in_jail: bool,
    /// The player's position around the board. 'Go' is at 0
    /// and 'Mayfair' (the last tile going clockwise) is at 35.
    pub position: u8,
    /// The amount of money the player has.
    pub balance: u16,
    /// The number of consecutive doubles the player has rolled.
    pub doubles_rolled: u8,
    /// A hashmap containing the indexes of the properties that
    /// the player owns in the form `HashMap<index, rent_level>`.
    pub property_rents: HashMap<usize, u8>,
}

impl Player {
    /// Create a vector of players.
    pub fn multiple_new(amount: usize) -> Vec<Player> {
        let mut players = Vec::with_capacity(amount);

        for i in 0..amount {
            players.push(Player {
                in_jail: false,
                position: 0,
                balance: 1500,
                doubles_rolled: 0,
                property_rents: HashMap::new(),
            })
        }

        players
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

/// The number of players playing the game. This should be between 2 and 4
/// inclusive, according to the rules. `State.hash()` is also built around
/// the fact that the maximum number of players is 4.
pub const NUM_PLAYERS: usize = 2;

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

    /// All the properties on the game board.
    pub static ref PROPERTIES: [Property; 22] = [
        Property::new(1, Color::Brown, 60, [70, 130, 220, 370, 750]),
        Property::new(3, Color::Brown, 60, [70, 130, 220, 370, 750]),
        Property::new(5, Color::LightBlue, 100, [80, 140, 240, 410, 800]),
        Property::new(6, Color::LightBlue, 100, [80, 140, 240, 410, 800]),
        Property::new(8, Color::LightBlue, 120, [100, 160, 260, 440, 860]),
        Property::new(10, Color::Pink, 140, [110, 180, 290, 460, 900]),
        Property::new(12, Color::Pink, 140, [110, 180, 290, 460, 900]),
        Property::new(13, Color::Pink, 160, [130, 200, 310, 490, 980]),
        Property::new(14, Color::Orange, 180, [140, 210, 330, 520, 1000]),
        Property::new(15, Color::Orange, 180, [140, 210, 330, 520, 1000]),
        Property::new(17, Color::Orange, 200, [160, 230, 350, 550, 1100]),
        Property::new(19, Color::Red, 220, [170, 250, 380, 580, 1160]),
        Property::new(21, Color::Red, 220, [170, 250, 380, 580, 1160]),
        Property::new(22, Color::Red, 240, [190, 270, 400, 610, 1200]),
        Property::new(23, Color::Yellow, 260, [200, 280, 420, 640, 1300]),
        Property::new(24, Color::Yellow, 260, [200, 280, 420, 640, 1300]),
        Property::new(26, Color::Yellow, 280, [220, 300, 440, 670, 1340]),
        Property::new(28, Color::Green, 300, [230, 320, 460, 700, 1400]),
        Property::new(30, Color::Green, 300, [230, 320, 460, 700, 1400]),
        Property::new(31, Color::Green, 320, [250, 340, 480, 730, 1440]),
        Property::new(33, Color::Blue, 350, [270, 360, 510, 740, 1500]),
        Property::new(35, Color::Blue, 400, [300, 400, 560, 810, 1600]),
    ];

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
