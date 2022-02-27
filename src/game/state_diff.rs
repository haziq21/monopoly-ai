use super::globals::*;
use std::collections::HashMap;

/*********        BRANCH TYPE        *********/

#[derive(Clone, Debug)]
/// The type of branch that led to a game state.
pub enum BranchType {
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
pub struct PropertyOwnership {
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

/// A field or property of a game state. There are 8 different fields (8 variants of this enum).
pub enum FieldDiff {
    /// The type of branch that led to a game state.
    BranchType(BranchType),
    /// The players playing the game.
    Players(Vec<Player>),
    /// The index of the player whose turn it currently is.
    CurrentPlayer(usize),
    /// A hashmap of properties owned by the players, with the
    /// keys being the position of a property around the board.
    OwnedProperties(HashMap<u8, PropertyOwnership>),
}

/*********        STATE DIFF        *********/

pub struct StateDiff {
    pub present_diffs: u8,
    /// Changes to the game state since the previous (parent) state.
    /// `FieldDiff`s in this vec will always appear in the same order:
    ///
    /// 0. `FieldDiff::BranchType`
    /// 1. `FieldDiff::Players`
    /// 2. `FieldDiff::CurrentPlayer`
    /// 3. `FieldDiff::OwnedProperties`
    pub diffs: Vec<FieldDiff>,
    pub parent: usize,
    pub children: Vec<usize>,
}

impl StateDiff {
    /*********        INITIALISATION INTERFACES        *********/

    /// Return a new `StateDiff` without any diff fields.
    pub fn new_with_parent(parent: usize) -> Self {
        StateDiff {
            diffs: vec![],
            present_diffs: 0,
            parent,
            children: vec![],
        }
    }

    /// Return a new `StateDiff` initialised to the root state of a game.
    pub fn new_root(player_count: usize) -> Self {
        Self {
            diffs: vec![
                FieldDiff::BranchType(BranchType::Choice),
                FieldDiff::Players(vec![Player::new(); player_count]),
                FieldDiff::CurrentPlayer(0),
                FieldDiff::OwnedProperties(HashMap::new()),
            ],
            present_diffs: 0b11110000,
            parent: 0,
            children: vec![],
        }
    }

    /*********        HELPERS        *********/

    /// Return whether the specified diff field is being tracked.
    pub fn diff_exists(&self, diff_id: u8) -> bool {
        (self.present_diffs >> diff_id) & 1 != 0
    }

    /// Return the index of the specified diff in `self.diffs` if it were to exist.
    pub fn get_supposed_diff_index(&self, diff_id: u8) -> usize {
        let relevant_bits = self.present_diffs >> diff_id;

        let high_bit_sum = (relevant_bits & 0b00000010)
            + (relevant_bits & 0b00000100)
            + (relevant_bits & 0b00001000)
            + (relevant_bits & 0b00010000)
            + (relevant_bits & 0b00100000)
            + (relevant_bits & 0b01000000)
            + (relevant_bits & 0b10000000);

        high_bit_sum.into()
    }

    /// Return the index of the specified diff in `self.diffs`,
    ///  or `None` if the state doesn't track it.
    pub fn get_diff_index(&self, diff_id: u8) -> Option<usize> {
        if self.diff_exists(diff_id) {
            return None;
        }

        Some(self.get_supposed_diff_index(diff_id))
    }

    /*********        DIFF SETTERS        *********/

    /// Insert the specified diff, or update it if it  
    /// already exists. Return a mutable reference to the diff.
    fn set_diff(&mut self, diff_id: u8, diff: FieldDiff) -> &mut FieldDiff {
        // Get the new index of the diff field
        let diff_index = self.get_supposed_diff_index(diff_id);

        if self.diff_exists(DIFF_ID_PLAYERS) {
            // Set the diff
            self.diffs[diff_index] = diff;
        } else {
            // Insert the diff
            self.diffs.insert(diff_index, diff);
            // Amend the diff presence flag
            self.present_diffs &= 1;
        }

        &mut self.diffs[diff_index]
    }

    /// Set a `BranchType` as the state's own diff.
    pub fn set_branch_type_diff(&mut self, branch_type: BranchType) {
        self.set_diff(DIFF_ID_BRANCH_TYPE, FieldDiff::BranchType(branch_type));
    }

    /// Clone a `players` reference, and set it as the
    /// state's own diff. Return a mutable reference to the modified diff.
    pub fn set_players_diff(&mut self, players: Vec<Player>) -> &mut Vec<Player> {
        match self.set_diff(DIFF_ID_PLAYERS, FieldDiff::Players(players)) {
            FieldDiff::Players(p) => p,
            _ => unreachable!(),
        }
    }

    /*********        OWNED DIFF HELPERS        *********/

    /// Return a mutable reference to the players diff.
    /// Panic if the state doesn't own the players diff.
    pub fn get_players_diff(&mut self) -> &mut Vec<Player> {
        let diff_index = match self.get_diff_index(DIFF_ID_PLAYERS) {
            Some(i) => i,
            None => {
                panic!("StateDiff.get_players_diff() called when the state doesn't own `players`")
            }
        };

        match &mut self.diffs[diff_index] {
            FieldDiff::Players(p) => p,
            _ => unreachable!(),
        }
    }
}
