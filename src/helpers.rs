use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};
use std::fmt;

#[derive(Debug, Copy, Clone)]
pub struct DiceRoll {
    pub probability: f64,
    pub sum: u8,
    pub is_double: bool,
}

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
pub enum ChanceCard {
    RentLvlTo1 = 1,
    RentLvlTo5 = 2,
    RentLvlIncForSet = 3,
    RentLvlDecForSet = 4,
    RentLvlIncForBoardSide = 5,
    RentLvlDecForBoardSide = 6,
    RentLvlDecForNeighbours = 7,
    BonusForYouAndOpponent = 8,
    SwapProperty = 9,
    SendOpponentToJail = 10,
    MoveToAnyProperty = 11,
}

pub struct Property {
    pub position: u8,
    pub color: Color,
    pub price: u16,
    pub rents: [u16; 5],
}

#[derive(Copy, Clone, Debug)]
pub enum StateType {
    Chance(f64),
    Choice,
}

impl StateType {
    pub fn probability(&self) -> f64 {
        match self {
            StateType::Chance(p) => *p,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Player {
    pub in_jail: bool,
    pub position: u8,
    pub balance: u16,
    pub doubles_rolled: u8,
    /// A hashmap containing the indexes of the properties that
    /// the player owns in the form `HashMap<index, rent_level>`
    pub property_rents: HashMap<usize, u8>,
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

pub fn build_property(position: u8, color: Color, price: u16, rents: [u16; 5]) -> Property {
    Property {
        position,
        color,
        price,
        rents,
    }
}

pub fn build_players(amount: usize) -> Vec<Player> {
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

// 2 to 4 players per game, according to the rules
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

    /// All the properties on the game board.
    pub static ref PROPERTIES: [Property; 22] = [
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
