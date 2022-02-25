use super::globals::*;
use super::Game;
use std::collections::HashMap;

type NodeSet<'a> = &'a Vec<StateDiff>;

/*********        BRANCH TYPE        *********/

#[derive(Clone, Debug)]
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

/// A field or property of a game state. There are 8 different fields (8 variants of this enum).
enum FieldDiff {
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
    /// Changes to the game state since the previous (parent) state.
    /// `FieldDiff`s in this vec will always appear in the same order (right to left):
    ///
    /// 0. `FieldDiff::BranchType`
    /// 1. `FieldDiff::Players`
    /// 2. `FieldDiff::CurrentPlayer`
    /// 3. `FieldDiff::OwnedProperties`
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

    pub fn gen_children(&self, all_nodes: NodeSet) -> Vec<StateDiff> {
        self.gen_chance_children(all_nodes)
    }

    /// Return child states that can be reached by rolling dice from this state.
    fn gen_chance_children(&self, all_nodes: NodeSet) -> Vec<StateDiff> {
        self.diff_branch_type(all_nodes);
        vec![]
    }

    /*********        STATE FIELD GETTERS        *********/

    /// Return the index of the specified diff in `self.diffs`,
    ///  or `None` if the state doesn't track it.
    fn get_diff_index(&self, diff_id: u8) -> Option<usize> {
        let relevant_bits = self.present_diffs >> diff_id;

        if relevant_bits & 1 == 1 {
            return None;
        }

        let high_bit_sum = (relevant_bits & 0b00000010)
            + (relevant_bits & 0b00000100)
            + (relevant_bits & 0b00001000)
            + (relevant_bits & 0b00010000)
            + (relevant_bits & 0b00100000)
            + (relevant_bits & 0b01000000)
            + (relevant_bits & 0b10000000);

        Some(high_bit_sum.into())
    }

    /// Return the branch type of the node.
    fn diff_branch_type<'g: 'r, 'r>(&'r self, all_nodes: NodeSet<'g>) -> &'r BranchType {
        match self.get_diff_index(0) {
            Some(i) => match &self.diffs[i] {
                FieldDiff::BranchType(a) => a,
                _ => unreachable!(),
            },
            None => all_nodes[self.parent].diff_branch_type(all_nodes),
        }
    }

    /*********        OTHER GETTERS        *********/
}
