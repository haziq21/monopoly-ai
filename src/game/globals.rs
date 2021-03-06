use lazy_static::lazy_static;
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs;

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
    pub fn unseen_counts(seen_cards: &[ChanceCard]) -> HashMap<ChanceCard, u8> {
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
    pub price: i32,
    /// The rent amount for each rent level of the property.
    /// `rents[0]` would be the rent amount for rent level 1,
    /// and `rents[4]` would be that of rent level 5.
    pub rents: [i32; 5],
}

impl Property {
    /// Creates a new property.
    pub fn new(color: Color, price: i32, rents: [i32; 5]) -> Property {
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
        self.position = JAIL_POSITION;
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

#[derive(Debug)]
pub struct GameplayStats {
    /// The net property worths of each player over time.
    property_worth: Vec<Vec<i32>>,
    /// The auction rates of each player over time.
    /// Each round looks like `(round, player, auctioned)`.
    auction_rate: Vec<(usize, usize, bool)>,
    /// The usage rate of location tiles. The tuple stores
    /// a fraction using its numerator and denominator.
    location_tile_usage: Vec<(u32, u32)>,
    /// The number of rounds that each player was in jail for.
    sentenced_rounds: Vec<u32>,
}

impl GameplayStats {
    /****     PUBLIC INTERFACES     ****/
    pub fn new(player_count: usize) -> GameplayStats {
        GameplayStats {
            sentenced_rounds: vec![0; player_count],
            property_worth: vec![],
            location_tile_usage: vec![(0, 0); player_count],
            auction_rate: vec![],
        }
    }

    pub fn update_location_tile_usage(&mut self, pindex: usize, used: bool) {
        self.location_tile_usage[pindex].0 += used as u32;
        self.location_tile_usage[pindex].1 += 1;
    }

    pub fn update_auction_rate(&mut self, pindex: usize, round: usize, auctioned: bool) {
        self.auction_rate.push((round, pindex, auctioned));
    }

    pub fn update_prop_worths(&mut self, worths: Vec<i32>) {
        self.property_worth.push(worths);
    }

    pub fn inc_sentenced_rounds(&mut self, pindex: usize) {
        self.sentenced_rounds[pindex] += JAIL_TRIES as u32;
    }

    pub fn save_to_csv(&self, loser: usize) {
        let uid: String = rand::thread_rng().gen::<u32>().to_string();
        println!("{:?}", fs::create_dir_all(format!("./data/{}", uid)));
        fs::write(
            format!("./data/{}/sentences.csv", uid),
            self.csv_sentenced_rounds(),
        );
        fs::write(
            format!("./data/{}/auctions.csv", uid),
            self.csv_auction_rate(),
        );
        fs::write(
            format!("./data/{}/prop_worth.csv", uid),
            self.csv_prop_worth(),
        );
        fs::write(format!("./data/{}/location.csv", uid), self.csv_location());
        fs::write(
            format!("./data/{}/loser.csv", uid),
            format!("loser\n{}", loser.to_string()),
        );
    }

    /****     HELPER FUNCTIONS     ****/

    fn get_player_count(&self) -> usize {
        self.sentenced_rounds.len()
    }

    fn get_round_count(&self) -> usize {
        self.property_worth.len()
    }

    fn csv_sentenced_rounds(&self) -> String {
        let headers = (0..self.sentenced_rounds.len())
            .map(|i| format!("player {}", i))
            .collect::<Vec<String>>()
            .join(",");

        let row = self
            .sentenced_rounds
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(",");

        [headers, row].join("\n")
    }

    fn csv_prop_worth(&self) -> String {
        let mut csv = "move number,".to_owned();
        csv.push_str(
            &(0..self.sentenced_rounds.len())
                .map(|i| format!("player {}", i))
                .collect::<Vec<String>>()
                .join(","),
        );

        for (i, row) in self.property_worth.iter().enumerate() {
            csv.push_str(&format!(
                "\n{},{}",
                i,
                row.iter()
                    .map(|j| j.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
            ));
        }

        csv
    }

    fn csv_location(&self) -> String {
        let headers = (0..self.sentenced_rounds.len())
            .map(|i| format!("player {}", i))
            .collect::<Vec<String>>()
            .join(",");

        let row = self
            .location_tile_usage
            .iter()
            .map(|x| {
                (if x.1 == 0 {
                    0.
                } else {
                    x.0 as f64 / x.1 as f64
                })
                .to_string()
            })
            .collect::<Vec<String>>()
            .join(",");

        [headers, row].join("\n")
    }

    fn csv_auction_rate(&self) -> String {
        let mut csv = "move number,player number,auctioned".to_owned();

        for row in &self.auction_rate {
            csv.push_str(&format!("\n{},{},{}", row.0, row.1, row.2 as u8));
        }

        csv
    }
}

#[derive(Copy, Clone)]
pub enum DiffID {
    Level1Rent = 1,
    SeenCcsHead,
    SeenCcs,
    OwnedProperties,
    CurrentPlayer,
    Players,
    JailRounds,
}

impl DiffID {
    pub fn all() -> [DiffID; 7] {
        [
            DiffID::Level1Rent,
            DiffID::SeenCcsHead,
            DiffID::SeenCcs,
            DiffID::OwnedProperties,
            DiffID::CurrentPlayer,
            DiffID::Players,
            DiffID::JailRounds,
        ]
    }
}

/// The position of 'Jail' on the game board.
pub const JAIL_POSITION: u8 = 9;
/// The position of 'Free parking' on the game board.
pub const FREE_PARKING_POSITION: u8 = 18;
/// The position of the 'Go to jail' tile on the game board.
pub const GO_TO_JAIL_POSITION: u8 = 27;
/// The total number of chance cards there are.
pub const TOTAL_CHANCE_CARDS: usize = 21;
/// Number of tries you can use to get out of jail before you have to pay.
pub const JAIL_TRIES: u8 = 3;

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
    pub static ref PROPS_BY_COLOR: HashMap<Color,HashSet<u8>> = HashMap::from([
        (Color::Brown, HashSet::from([1, 3])),
        (Color::LightBlue, HashSet::from([5, 6, 8])),
        (Color::Pink, HashSet::from([10, 12, 13])),
        (Color::Orange, HashSet::from([14, 15, 17])),
        (Color::Red, HashSet::from([19, 21, 22])),
        (Color::Yellow, HashSet::from([23, 24, 26])),
        (Color::Green, HashSet::from([28, 30, 31])),
        (Color::Blue, HashSet::from([33, 35])),
    ]);

    /// Positions of the properties on the game board, sorted by the side of the board they're on.
    pub static ref PROPS_BY_SIDE: [HashSet<u8>; 4] = [
        HashSet::from([1, 3, 5, 6, 8]),
        HashSet::from([10, 12, 13, 14, 15, 17]),
        HashSet::from([19, 21, 22, 23, 24, 26]),
        HashSet::from([28, 30, 31, 33, 35])
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

/// From the set of {x ??? Z | 0 ??? x ??? n }, return all the possible k-long combinations.
/// Adapted from this stackoverflow answer (https://stackoverflow.com/a/8332722) written in Delphi.
pub fn get_combinations(n: usize, k: usize) -> Vec<Vec<usize>> {
    let mut curr_comb = vec![];
    let mut result = vec![];

    // Setup comb for the initial combination
    for i in 0..k {
        curr_comb.push(i);
    }

    result.push(curr_comb.clone());

    loop {
        let mut i = k - 1;
        curr_comb[i] += 1;

        while i > 0 && curr_comb[i] >= n - k + 1 + i {
            i -= 1;
            curr_comb[i] += 1;
        }

        // Combination (n-k, n-k+1, ..., n) reached
        if curr_comb[0] > n - k {
            // No more combinations can be generated
            break;
        }

        // comb now looks like (..., x, n, n, n, ..., n).
        // Turn it into (..., x, x + 1, x + 2, ...)
        for j in (i + 1)..k {
            curr_comb[j] = curr_comb[j - 1] + 1;
        }

        result.push(curr_comb.clone());
    }

    result
}
