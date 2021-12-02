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
    pub balance: u16,
    /// The number of consecutive doubles the player has rolled.
    pub doubles_rolled: u8,
}

impl Player {
    /// Create a vector of players.
    pub fn multiple_new(amount: usize) -> Vec<Player> {
        let mut players = Vec::with_capacity(amount);

        for _ in 0..amount {
            players.push(Player {
                in_jail: false,
                position: 0,
                balance: 1500,
                doubles_rolled: 0,
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
