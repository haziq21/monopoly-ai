use lazy_static::lazy_static;
use std::collections::HashSet;

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

pub enum ChanceCard {
    RentLvlTo1,
    RentLvlTo5,
    RentLvlIncForSet,
    RentLvlDecForSet,
    RentLvlIncForBoardSide,
    RentLvlDecForBoardSide,
    RentLvlDecForNeighbours,
    BonusForYouAndOpponent,
    SwapProperty,
    SendOpponentToJail,
    MoveToAnyProperty,
}

pub struct Property {
    pub position: u8,
    pub color: Color,
    pub price: u16,
    pub rents: [u16; 5],
}

pub fn build_property(position: u8, color: Color, price: u16, rents: [u16; 5]) -> Property {
    Property {
        position,
        color,
        price,
        rents,
    }
}

pub const CC_POSITIONS: HashSet<u8> = HashSet::from([2, 4, 11, 20, 29, 32]);

lazy_static! {
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
