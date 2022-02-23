use super::globals::*;
use std::collections::HashMap;

/*********        BRANCH TYPE        *********/

#[derive(Copy, Clone, Debug)]
/// The type of branch that led to a game state.
enum BranchType {
    /// A game state that was achieved by chance (e.g. by rolling the dice / getting a chance card).
    /// The associated value is the probability of the chance.
    Chance(f64),
    /// A game state that was achieved by making a choice.
    Choice,
}

impl BranchType {
    /// Return the associated value if `self` is
    /// a `BranchType::Chance`, and panic otherwise.
    pub fn probability(&self) -> f64 {
        match self {
            BranchType::Chance(p) => *p,
            _ => unreachable!(),
        }
    }
}

/*********        PROPERTY OWNERSHIP        *********/

#[derive(Copy, Clone, Debug)]
/// Information about a property related to its ownership.
struct PropertyOwnership {
    /// The index of the player who owns this property
    pub owner: usize,
    /// The rent level of this property.
    /// Rent level starts at 1 and caps out at 5.
    pub rent_level: u8,
}

impl PropertyOwnership {
    /// Raise the rent level by one, if possible. Return whether this had any effect.
    pub fn raise_rent(&mut self) -> bool {
        if self.rent_level < 5 {
            self.rent_level += 1;
            return true;
        }

        false
    }

    /// Lower the rent level by one, if possible. Return whether this had any effect.
    pub fn lower_rent(&mut self) -> bool {
        if self.rent_level > 1 {
            self.rent_level -= 1;
            return true;
        }

        false
    }
}

/*********        FIELD DIFF        *********/

/// A field or property of a game state.
enum FieldDiff {
    /// The type of branch that led to a game state.
    BranchType(BranchType),
    /// The players playing the game.
    Players(Vec<Player>),
    /// A hashmap of properties owned by the players, with the
    /// keys being the position of a property around the board.
    OwnedProperties(HashMap<u8, PropertyOwnership>),
}

/*********        STATE DIFF        *********/

pub struct StateDiff {
    diffs: Vec<FieldDiff>,
    present_diffs: u8,
    parent: usize,
    children: Vec<usize>,
}

impl StateDiff {
    /*********        PUBLIC INTERFACES        *********/

    /// Return a new `StateDiff` initialised to the root state of a game.
    pub fn new(player_count: usize) -> Self {
        Self {
            diffs: vec![],
            present_diffs: 0b11111111,
            parent: 0,
            children: vec![],
        }
    }
}
