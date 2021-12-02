mod details;
pub use details::*;

use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};

/// The number of factors that contribute to a state's static evaluation.
pub const NUM_FACTORS: usize = 6;

/// The number of players playing the game. This should
/// be between 2 and 4 inclusive, according to the rules.
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
