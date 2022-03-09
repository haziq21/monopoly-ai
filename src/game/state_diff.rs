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

/*********        MOVE TYPE        *********/

#[derive(Debug, Clone)]
pub enum MoveType {
    Undefined,
    Roll,
    Property,
    Location,
    ChanceCard,
    ChoicefulCC(ChanceCard),
}

impl MoveType {
    pub fn when_landed_on(tile: u8) -> MoveType {
        if PROP_POSITIONS.contains(&tile) {
            MoveType::Property
        } else if CC_POSITIONS.contains(&tile) {
            MoveType::ChanceCard
        } else if LOC_POSITIONS.contains(&tile) {
            MoveType::Location
        } else {
            MoveType::Roll
        }
    }
}

/*********        FIELD DIFF        *********/

/// A field or property of a game state. There are 8 different fields (8 variants of this enum).
#[derive(Debug, Clone)]
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
    /// The chance cards that have been used, ordered from least recent to most recent.
    SeenCCs(Vec<ChanceCard>),
    /// The starting index of `SeenCCs`.
    SeenCCsHead(usize),
    /// The number of rounds to go before the effect of the chance card
    /// "all players pay level 1 rent for the next two rounds" wears off.
    Level1Rent(u8),
}

/*********        STATE DIFF        *********/

#[derive(Debug, Clone)]
pub struct StateDiff {
    pub present_diffs: u8,
    /// Changes to the game state since the previous (parent) state.
    /// `FieldDiff`s in this vec will always appear in the same order:
    ///
    /// 0. `FieldDiff::BranchType`
    /// 1. `FieldDiff::Players`
    /// 2. `FieldDiff::CurrentPlayer`
    /// 3. `FieldDiff::OwnedProperties`
    /// 4. `FieldDiff::SeenCCs`
    /// 5. `FieldDiff::SeenCCsHead`
    pub diffs: Vec<FieldDiff>,
    pub parent: usize,
    pub children: Vec<usize>,
    /// The type of move to be made after a state.
    /// This is not in `diffs` as it changes every move.
    pub next_move: MoveType,
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
            next_move: MoveType::Undefined,
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
                FieldDiff::SeenCCs(vec![]),
                FieldDiff::SeenCCsHead(0),
                FieldDiff::Level1Rent(0),
            ],
            present_diffs: 0b11111110,
            parent: 0,
            children: vec![],
            next_move: MoveType::Roll,
        }
    }

    /*********        HELPERS        *********/

    /// Return whether the specified diff field is being tracked.
    pub fn diff_exists(&self, diff_id: u8) -> bool {
        (self.present_diffs >> diff_id) & 1 == 1
    }

    /// Return the index of the specified diff in `self.diffs` if it were to exist.
    pub fn get_supposed_diff_index(&self, diff_id: u8) -> usize {
        let relevant_bits = self.present_diffs >> diff_id;

        let high_bit_sum = (relevant_bits >> 1 & 1)
            + (relevant_bits >> 2 & 1)
            + (relevant_bits >> 3 & 1)
            + (relevant_bits >> 4 & 1)
            + (relevant_bits >> 5 & 1)
            + (relevant_bits >> 6 & 1)
            + (relevant_bits >> 7 & 1);

        high_bit_sum.into()
    }

    /// Return the index of the specified diff in `self.diffs`,
    ///  or `None` if the state doesn't track it.
    pub fn get_diff_index(&self, diff_id: u8) -> Option<usize> {
        if !self.diff_exists(diff_id) {
            return None;
        }

        Some(self.get_supposed_diff_index(diff_id))
    }

    /// Insert the specified diff, or update it if it  
    /// already exists. Return a mutable reference to the diff.
    fn set_diff(&mut self, diff_id: u8, diff: FieldDiff) -> &mut FieldDiff {
        // Get the new index of the diff field
        let diff_index = self.get_supposed_diff_index(diff_id);

        if self.diff_exists(diff_id) {
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

    /*********        DIFF SETTERS        *********/

    /// Set a `BranchType` as the state's own diff.
    pub fn set_branch_type_diff(&mut self, branch_type: BranchType) {
        self.set_diff(DIFF_ID_BRANCH_TYPE, FieldDiff::BranchType(branch_type));
    }

    /// Set a `players` vector as the state's own diff.
    /// Return a mutable reference to the modified diff.
    pub fn set_players_diff(&mut self, players: Vec<Player>) -> &mut Vec<Player> {
        match self.set_diff(DIFF_ID_PLAYERS, FieldDiff::Players(players)) {
            FieldDiff::Players(p) => p,
            _ => unreachable!(),
        }
    }

    pub fn set_current_player_diff(&mut self, curr_player: usize) {
        self.set_diff(
            DIFF_ID_CURRENT_PLAYER,
            FieldDiff::CurrentPlayer(curr_player),
        );
    }

    pub fn set_owned_properties_diff(&mut self, owned_properties: HashMap<u8, PropertyOwnership>) {
        self.set_diff(
            DIFF_ID_OWNED_PROPERTIES,
            FieldDiff::OwnedProperties(owned_properties),
        );
    }

    /// Set a `seen_ccs` vector as the state's own diff.
    pub fn set_seen_ccs_diff(&mut self, seen_ccs: Vec<ChanceCard>) {
        self.set_diff(DIFF_ID_SEEN_CCS, FieldDiff::SeenCCs(seen_ccs));
    }

    pub fn set_seen_ccs_head_diff(&mut self, seen_ccs_head: usize) {
        self.set_diff(DIFF_ID_SEEN_CCS_HEAD, FieldDiff::SeenCCsHead(seen_ccs_head));
    }

    pub fn set_level_1_rent_diff(&mut self, rent: u8) {
        self.set_diff(DIFF_ID_LEVEL_1_RENT, FieldDiff::Level1Rent(rent));
    }
}
