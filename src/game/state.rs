#![allow(dead_code, unused_mut, unused_variables)]

use std::collections::HashMap;
use std::fmt;

use super::globals::*;

#[derive(Copy, Clone, Debug)]
/// One of two types that a game state could be.
enum StateType {
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

#[derive(Copy, Clone, Debug)]
/// Information about a property related to its ownership.
struct OwnedProperty {
    /// The index of the player who owns this property
    pub owner: usize,
    /// The rent level of this property.
    /// Rent level starts at 1 and caps out at 5.
    pub rent_level: u8,
}

impl OwnedProperty {
    /// Raise the rent level by one, if possible. Return whether this had any effect.
    pub fn raise_rent(&mut self) -> bool {
        if self.rent_level < 5 {
            self.rent_level += 1;
            true
        } else {
            false
        }
    }

    /// Lower the rent level by one, if possible. Return whether this had any effect.
    pub fn lower_rent(&mut self) -> bool {
        if self.rent_level > 1 {
            self.rent_level -= 1;
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Debug)]
/// The state of a game as a node on the game tree.
pub struct State {
    /// The type of the state - either chance or choice.
    r#type: StateType,
    /// The players playing the game
    players: Vec<Player>,
    /// A hashmap of properties owned by the players, with the
    /// keys being the position of a property around the board.
    owned_properties: HashMap<u8, OwnedProperty>,
    /// The index of the player (from `players`) whose turn it currently is.
    current_player_index: usize,
    /// Whether the child states are achievable through chance
    /// (dice rolling) or choice (players making decisions).
    next_move_is_chance: bool,
    /// The choiceful chance card that a player needs to act on in child states.
    /// This being `Some(_)` implies that `next_move_is_chance == false`.
    active_cc: Option<ChanceCard>,
    /// The number of rounds to go before the effect of the chance card
    /// "all players pay level 1 rent for the next two rounds" wears off.
    lvl1rent_cc: u8,
    /// The chance cards that have been used, ordered from least recent to most recent.
    seen_ccs: Vec<ChanceCard>,
    /// The child nodes of this state.
    pub children: Vec<Box<State>>,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut metadata = match self.r#type {
            StateType::Chance(p) => format!("Probability: \x1b[33m{:.2}%\x1b[0m", p * 100.),
            StateType::Choice => String::from("No probability"),
        };

        match self.active_cc {
            Some(cc) => metadata.push_str(&format!("\nActive CC: {:?}", cc)),
            None => (),
        }

        if self.lvl1rent_cc > 0 {
            metadata += &format!("\nLvl1rent: {} turns to go", self.lvl1rent_cc);
        }

        let mut players_str = "".to_owned();
        for i in 0..self.players.len() {
            players_str += &format!("{}", self.players[i]);

            if self.current_player_index == i {
                players_str += &format!(
                    " < next: \x1b[36m{}\x1b[0m",
                    if self.next_move_is_chance {
                        "chance"
                    } else {
                        "choice"
                    }
                )
            }

            players_str += "\n";
        }

        write!(f, "{}\n{}", metadata, players_str)
    }
}

impl State {
    /*********        PUBLIC INTERFACES        *********/

    /// Create a new game state with `player_count` players.
    pub fn new(player_count: usize) -> State {
        State {
            r#type: StateType::Choice,
            players: Player::multiple_new(player_count),
            owned_properties: HashMap::new(),
            current_player_index: 0,
            next_move_is_chance: true,
            active_cc: None,
            lvl1rent_cc: 0,
            seen_ccs: vec![],
            children: vec![],
        }
    }

    /// Calculates and stores `state.children` (child nodes of `state` on the game tree).
    pub fn find_children(&mut self) {
        if self.children.len() == 0 {
            let children = if self.next_move_is_chance {
                self.roll_effects()
            } else {
                self.choice_effects()
            };

            self.children = children;
        }
    }

    /// Return the outcome of an MCTS rollout from this state.
    pub fn rollout(&self) -> u8 {
        0
    }

    /*********        ALIASES (FOR CONVENIECE)        *********/

    /// A mutable reference to the player whose turn it currently is.
    fn current_player(&mut self) -> &mut Player {
        &mut self.players[self.current_player_index]
    }

    /// The position of the current player.
    fn current_position(&self) -> u8 {
        self.players[self.current_player_index].position
    }

    /// The property the current player is on.
    fn current_property(&self) -> Option<&Property> {
        PROPERTIES.get(&self.current_position())
    }

    /// A mutable reference to the owned property the current player is on.
    fn current_owned_property(&mut self) -> Option<&mut OwnedProperty> {
        self.owned_properties.get_mut(&self.current_position())
    }

    /// The owned property located at `key`.
    fn owned_prop(&mut self, key: &u8) -> Option<&mut OwnedProperty> {
        self.owned_properties.get_mut(key)
    }

    /*********        HELPER FUNCTIONS        *********/

    /// Move the current player by the specified amount of tiles.
    fn move_by(&mut self, amount: u8) {
        let new_pos = (self.current_player().position + amount) % 36;

        // Set the player's `in_jail` flag to false if appropriate
        if self.current_player().in_jail && amount != 0 {
            self.current_player().in_jail = false;
        }

        // Give the player $200 if they pass 'Go'
        if new_pos < self.current_player().position {
            self.current_player().balance += 200;
        }

        // Update the position
        self.current_player().position = new_pos;
    }

    /// Change the `current_player_index` to the index of the player
    /// whose turn it is next, but only if the current player didn't
    /// roll doubles (in which case it would be their turn again).
    fn setup_next_player(&mut self) {
        // This player didn't roll doubles in their previous turn
        if self.current_player().doubles_rolled == 0 {
            // Change the current_player_index to the index of the next player
            self.current_player_index = (self.current_player_index + 1) % NUM_PLAYERS;
        }

        // The player whose turn it is next rolls the dice
        self.next_move_is_chance = true;
    }

    /// Send a player to jail.
    fn send_to_jail(&mut self, player_index: usize) {
        // Set the player's position to jail
        self.players[player_index].position = 9;
        self.players[player_index].in_jail = true;

        // Reset the doubles counter
        self.players[player_index].doubles_rolled = 0;
    }

    /// Send the current player to jail.
    fn send_current_to_jail(&mut self) {
        self.send_to_jail(self.current_player_index)
    }

    fn or_choice_clone(&self, children: Vec<Box<State>>) -> Vec<Box<State>> {
        if children.len() > 0 {
            children
        } else {
            vec![Box::new(self.clone_to_choice())]
        }
    }

    /// Return a clone of `self` with the state type as `StateType::Choice`.
    fn clone_to_choice(&self) -> State {
        State {
            r#type: StateType::Choice,
            ..self.clone()
        }
    }

    /// Return every possible result of attempting to roll doubles for a maximum of `tries` times.
    fn roll_for_doubles(tries: i32) -> Vec<DiceRoll> {
        /*
         *  Let P(S) be the probability that a double is not attained in one roll.
         *  Let P(r) be the probability of obtaining a specific dice configuration
         *  `r` after one roll. `SIGNIFICANT_ROLLS` demonstrates all possible
         *  "specific dice configurations".
         *
         *  When rolling the dice for maximum of `n` times, or stopping
         *  when we get doubles, the probabilities work out as follows:
         *
         *  The probability of the final roll `r` being any double `d` (where the sum
         *  of the dice is `2d`) is given by `sum_(i=0)^(n-1) P(r) * P(S)^i`.
         *
         *  The probability of all `n` rolls being non-doubles (and hence the
         *  final roll being a non-double `r`) is given by `P(r) * P(S)^(n - 1)`.
         *
         *  The following code implements this.
         */
        SIGNIFICANT_ROLLS
            .iter()
            .map(|&roll| {
                DiceRoll {
                    probability: if roll.is_double {
                        let mut double_probability = 0.;

                        // sum_(i=0)^(n-1) P(r) * P(S)^i
                        for i in 0..tries {
                            double_probability += roll.probability * SINGLE_PROBABILITY.powi(i);
                        }

                        double_probability
                    } else {
                        // P(r) * P(S)^(n - 1)
                        roll.probability * SINGLE_PROBABILITY.powi(tries - 1)
                    },
                    ..roll
                }
            })
            .collect()
    }

    /*********        CHOICEFUL CHANCE CARD EFFECTS        *********/

    fn cc_rent_level_to(&self, n: u8) -> Vec<Box<State>> {
        let mut children = vec![];

        for (pos, prop) in &self.owned_properties {
            // "RentLvlTo5" only applies to your properties (not opponents)
            if n == 5 && prop.owner != self.current_player_index {
                continue;
            }

            // Don't need to add another child node if the rent level is already at its max/min
            if prop.rent_level != n {
                let mut child = self.clone_to_choice();
                child.owned_properties.get_mut(&pos).unwrap().rent_level = n;
                children.push(Box::new(child));
            }
        }

        self.or_choice_clone(children)
    }

    fn cc_rent_change_for_set(&self, increase: bool) -> Vec<Box<State>> {
        let mut children = vec![];

        // Loop through all the color sets
        for (_, positions) in PROPS_BY_COLOR.iter() {
            let mut new_state = self.clone_to_choice();
            let mut has_effect = false;

            // Loop through all the properties in this color set
            for pos in positions {
                if let Some(prop) = new_state.owned_properties.get_mut(&pos) {
                    if increase {
                        has_effect |= prop.raise_rent();
                    } else {
                        has_effect |= prop.lower_rent();
                    }
                }
            }

            // Only store the new state if it's different
            if has_effect {
                children.push(Box::new(new_state));
            }
        }

        self.or_choice_clone(children)
    }

    fn cc_rent_change_for_side(&self, increase: bool) -> Vec<Box<State>> {
        // Possible child states, in clockwise order of affected area.
        // E.g. children[0] is the state where the first side of the board is
        // affected and children[3] is the state where the last side is affected.
        let mut children = vec![Box::new(self.clone_to_choice()); 4];

        // Bitmap of whether the states in `children` are any different from `self`.
        // Rightmost bit indicates whether `children[0]` is different from `self` and
        // fourthmost bit from the right indicates whether `children[3]` is different.
        let mut has_effect: u8 = 0;

        // Loop through the positions of all the owned properties
        for pos in self.owned_properties.keys() {
            // The side of the board `pos` is on - 0 is the first side (with 'go'
            // and 'jail') and 3 is the last side (with 'go to jail' and 'go')
            let i = pos / 9;
            let prop = children[i as usize].owned_properties.get_mut(&pos).unwrap();
            let changed = if increase {
                prop.raise_rent()
            } else {
                prop.lower_rent()
            };

            // Update the bitmap accordingly
            has_effect |= (changed as u8) << i;
        }

        // Remove the states that didn't have an effect
        for c in (0..4).rev() {
            if has_effect & (1 << c) == 0 {
                children.swap_remove(c);
            }
        }

        self.or_choice_clone(children)
    }

    fn cc_rent_dec_for_neighbours(&self) -> Vec<Box<State>> {
        let mut children = vec![];

        for (pos, prop) in &self.owned_properties {
            // Skip if this property isn't owned by the current player
            if prop.owner != self.current_player_index {
                continue;
            }

            let mut new_state = self.clone_to_choice();
            let mut has_effect = false;

            // Raise this property's rent level
            has_effect |= new_state
                .owned_properties
                .get_mut(&pos)
                .unwrap()
                .raise_rent();

            // Lower neighbours' rent levels (if they're owned)
            for n_pos in PROPERTY_NEIGHBOURS[&pos] {
                if let Some(n_prop) = new_state.owned_properties.get_mut(&n_pos) {
                    has_effect |= n_prop.lower_rent();
                }
            }

            // Store new state if it's different
            if has_effect {
                children.push(Box::new(new_state));
            }
        }

        self.or_choice_clone(children)
    }

    fn cc_bonus(&self) -> Vec<Box<State>> {
        let mut children = vec![];

        for i in 0..self.players.len() {
            // Skip the current player
            if i == self.current_player_index {
                continue;
            }

            let mut new_state = self.clone_to_choice();

            // Award $200 bonus to this player
            new_state.current_player().balance += 200;

            // Award $200 bonus to an opponent
            new_state.players[i].balance += 200;

            children.push(Box::new(new_state));
        }

        // No need for `self.children_or_clone(children)`
        // because we know there's at least one opponent
        children
    }

    fn cc_swap_property(&self) -> Vec<Box<State>> {
        let mut children = vec![];
        let mut my_props = vec![];
        let mut opponent_props = vec![];

        // Loop through all owned properties to sort them out by ownership
        for (&pos, prop) in &self.owned_properties {
            if prop.owner == self.current_player_index {
                my_props.push(pos);
            } else {
                opponent_props.push(pos);
            }
        }

        // Loop through all the sorted properties
        for my_pos in my_props {
            for opponent_pos in &opponent_props {
                let mut new_state = self.clone_to_choice();

                // Get the owners of the properties
                let opponent = new_state.owned_properties[&opponent_pos].owner;
                let me = new_state.owned_properties[&my_pos].owner;
                // Swap properties
                new_state.owned_prop(&my_pos).unwrap().owner = opponent;
                new_state.owned_prop(&opponent_pos).unwrap().owner = me;

                children.push(Box::new(new_state));
            }
        }

        // No need for `state.children_or_clone(children)`
        // because we know there's at least one opponent
        children
    }

    fn cc_opponent_to_jail(&self) -> Vec<Box<State>> {
        let mut children = vec![];

        for i in 0..self.players.len() {
            // Skip the current player
            if i == self.current_player_index {
                continue;
            }

            // Send the opponent to jail
            let mut new_state = self.clone_to_choice();
            new_state.send_to_jail(i);

            children.push(Box::new(new_state));
        }

        // No need for `state.children_or_clone(children)`
        // because we know there's at least one opponent
        children
    }

    fn cc_move_to_any_property(&self) -> Vec<Box<State>> {
        let mut children = vec![];

        for &pos in PROP_POSITIONS.iter() {
            let mut new_state = self.clone_to_choice();

            // Player can move to any property on the board
            new_state.current_player().position = pos;
            // Effects of landing on the property
            children.splice(children.len().., self.prop_choice_effects());
        }

        children
    }

    /*********        STATE GENERATION        *********/

    /// This function requires access to the player weights (for auctioning),
    /// which is why state generation is implemented on `Game` rather than on `State`.
    fn prop_full_effects(&self) -> Vec<Box<State>> {
        let current_pos = self.current_position();

        if let Some(prop) = self.owned_properties.get(&current_pos) {
            let mut new_state = self.clone_to_choice();

            // The current player owes rent to the owner of this property
            if prop.owner != new_state.current_player_index {
                let balance_due = if new_state.lvl1rent_cc > 0 {
                    PROPERTIES[&current_pos].rents[0]
                } else {
                    PROPERTIES[&current_pos].rents[prop.rent_level as usize - 1]
                };

                // Pay the owner...
                new_state.players[prop.owner].balance += balance_due;
                // ...using the current player's money
                new_state.current_player().balance -= balance_due;
            }

            // Raise the rent level
            new_state.current_owned_property().unwrap().raise_rent();

            // It's the end of this player's turn
            new_state.setup_next_player();

            vec![Box::new(new_state)]
        } else {
            // Choose to auction this property
            // let mut no_buy = state.clone_to_choice();
            // let auction_winner = self.auction_winner();
            // no_buy.players[auction_winner].balance -= PROPERTIES[&current_pos].price;
            // no_buy.owned_properties.insert(
            //     current_pos,
            //     OwnedProperty {
            //         owner: auction_winner,
            //         rent_level: 1,
            //     },
            // );

            // // Choose to buy this property
            // let mut buy_prop = state.clone_to_choice();
            // buy_prop.current_player().balance -= PROPERTIES[&current_pos].price;
            // buy_prop.owned_properties.insert(
            //     current_pos,
            //     OwnedProperty {
            //         owner: buy_prop.current_player_index,
            //         rent_level: 1,
            //     },
            // );

            vec![]
        }
    }

    /// Return child nodes of a game state that can be reached from a location tile.
    fn loc_choice_effects(&self) -> Vec<Box<State>> {
        let mut children = vec![];

        for &pos in PROP_POSITIONS.iter() {
            let mut new_state = self.clone_to_choice();

            // Play $100
            new_state.current_player().balance -= 100;
            // Player can teleport to any property on the board
            new_state.current_player().position = pos;
            // Effects of landing on the property
            children.splice(children.len().., self.prop_full_effects());
        }

        // There's also the option to do nothing
        children.push(Box::new(self.clone_to_choice()));

        children
    }

    /// Return child nodes of a game state that can be reached by buying or auctioning a property
    fn prop_choice_effects(&self) -> Vec<Box<State>> {
        let current_pos = self.current_position();

        // Choose to auction this property
        let mut no_buy = self.clone_to_choice();
        let auction_winner = 0; //self.auction_winner();
        no_buy.players[auction_winner].balance -= PROPERTIES[&current_pos].price;
        no_buy.owned_properties.insert(
            current_pos,
            OwnedProperty {
                owner: auction_winner,
                rent_level: 1,
            },
        );

        // Choose to buy this property
        let mut buy_prop = self.clone_to_choice();
        buy_prop.current_player().balance -= PROPERTIES[&buy_prop.current_position()].price;
        buy_prop.owned_properties.insert(
            buy_prop.current_position(),
            OwnedProperty {
                owner: buy_prop.current_player_index,
                rent_level: 1,
            },
        );

        vec![Box::new(no_buy), Box::new(buy_prop)]
    }

    /// Return child nodes of the current game state that can be
    /// reached by making a decision from a chance card tile.
    fn cc_choice_effects(&self) -> Vec<Box<State>> {
        let mut children = match self.active_cc.unwrap() {
            ChanceCard::RentLvlTo1 => self.cc_rent_level_to(1),
            ChanceCard::RentLvlTo5 => self.cc_rent_level_to(5),
            ChanceCard::RentLvlIncForSet => self.cc_rent_change_for_set(true),
            ChanceCard::RentLvlDecForSet => self.cc_rent_change_for_set(false),
            ChanceCard::RentLvlIncForBoardSide => self.cc_rent_change_for_side(true),
            ChanceCard::RentLvlDecForBoardSide => self.cc_rent_change_for_side(false),
            ChanceCard::RentLvlDecForNeighbours => self.cc_rent_dec_for_neighbours(),
            ChanceCard::BonusForYouAndOpponent => self.cc_bonus(),
            ChanceCard::SwapProperty => self.cc_swap_property(),
            ChanceCard::SendOpponentToJail => self.cc_opponent_to_jail(),
            ChanceCard::MoveToAnyProperty => self.cc_move_to_any_property(),
        };

        // Reset the active chance card
        for child in &mut children {
            child.active_cc = None;
        }

        children
    }

    /// Return child nodes of the current game state that can be reached by making a choice.
    fn choice_effects(&self) -> Vec<Box<State>> {
        // The player landed on a location tile
        let mut children = if LOC_POSITIONS.contains(&self.current_position()) {
            self.loc_choice_effects()
        }
        // The player landed on a property tile
        else if PROP_POSITIONS.contains(&self.current_position()) {
            self.prop_choice_effects()
        }
        // The player landed on a chance card tile
        else if CC_POSITIONS.contains(&self.current_position()) {
            self.cc_choice_effects()
        } else {
            unreachable!(); // Just in case
        };

        for child in &mut children {
            child.setup_next_player();
        }

        children
    }

    /// Return child nodes of the current game state that
    /// can be reached by rolling to a chance card tile.
    /// This modifies `self` and is only called in `roll_effects()`.
    fn cc_chance_effects(&mut self) -> Vec<Box<State>> {
        let mut children = vec![];
        let unit_probability = self.r#type.probability() / 21.;

        // Chance card: -$50 per property owned
        let mut property_penalty = State {
            r#type: StateType::Chance(unit_probability),
            ..self.clone()
        };
        let mut property_penalty_deduction = 0;

        // Deduct $50 per property owned
        for (_, prop) in &property_penalty.owned_properties {
            if prop.owner == property_penalty.current_player_index {
                property_penalty_deduction += 50;
            }
        }
        // Only add a new child state if it's different
        if property_penalty_deduction > 0 {
            property_penalty.current_player().balance -= property_penalty_deduction;
            property_penalty.setup_next_player();
            children.push(Box::new(property_penalty));
        }

        // Chance card: Pay level 1 rent for 2 rounds
        children.push(Box::new(State {
            r#type: StateType::Chance(unit_probability),
            lvl1rent_cc: (self.players.len() * 2) as u8 + 1,
            ..self.clone()
        }));

        // The chance card "Move all players not in jail to free parking"
        // is implemented in `roll_effects()` for optimisation purposes.

        // Chance cards that require the player to make a choice
        let choiceful_ccs = [
            (3, ChanceCard::RentLvlTo1),
            (1, ChanceCard::RentLvlTo5),
            (3, ChanceCard::RentLvlIncForSet),
            (1, ChanceCard::RentLvlDecForSet),
            (1, ChanceCard::RentLvlIncForBoardSide),
            (1, ChanceCard::RentLvlDecForBoardSide),
            (2, ChanceCard::RentLvlDecForNeighbours),
            (2, ChanceCard::BonusForYouAndOpponent),
        ];

        // Push the child states for all the choiceful chance cards
        for (amount, card) in choiceful_ccs {
            children.push(Box::new(State {
                r#type: StateType::Chance(unit_probability * amount as f64),
                active_cc: Some(card),
                next_move_is_chance: false,
                ..self.clone()
            }));
        }

        let total_children_probability = children
            .iter()
            .map(|child| child.r#type.probability())
            .sum::<f64>()
            // Add this probability to account for the "all players to jail"
            // chance card that's implemented in `roll_effects()`
            + unit_probability;

        // Correct the current state's probability to
        // account for the chance card child states
        if let StateType::Chance(p) = self.r#type {
            self.r#type = StateType::Chance(p - total_children_probability);
        };

        // total_children_probability != state.r#type.probability() when at
        // least one choiceless chance card has no effect. So `state.probability() == 0`
        // when every chance card has an effect.

        // `state` is the state where none of the chance cards apply, so it's the next player's turn.
        self.setup_next_player();

        children
    }

    fn in_place_prop_chance_effects(&mut self) {
        let current_pos = self.current_position();

        if let Some(prop) = self.owned_properties.get(&current_pos) {
            // The current player owes rent to the owner of this property
            if prop.owner != self.current_player_index {
                let balance_due = if self.lvl1rent_cc > 0 {
                    PROPERTIES[&current_pos].rents[0]
                } else {
                    PROPERTIES[&current_pos].rents[prop.rent_level as usize - 1]
                };

                // Pay the owner...
                self.players[prop.owner].balance += balance_due;
                // ...using the current player's money
                self.current_player().balance -= balance_due;
            }

            // Raise the rent level
            self.current_owned_property().unwrap().raise_rent();

            // It's the end of this player's turn
            self.setup_next_player();
        } else {
            // The player has to decide whether to buy or auction
            self.next_move_is_chance = false;
        }
    }

    /// Return child nodes of the current game state that can be reached by rolling dice.
    fn roll_effects(&self) -> Vec<Box<State>> {
        let mut children = vec![];

        // Probability of rolling (without doubles) to a chance
        // card tile and getting the "all to parking" card
        let mut atp_singles_probability = 0.;
        // Probability of rolling (with doubles) to a chance
        // card tile and getting the "all to parking" card
        let mut atp_doubles_probability = 0.;

        // Performs some other actions before pushing `state` to `children`
        let mut push_state = |mut s: State, rolled_doubles: bool| {
            let current_pos = s.current_position();
            // Player landed on a property tile
            if PROP_POSITIONS.contains(&current_pos) {
                s.in_place_prop_chance_effects();
            }
            // Player landed on a chance card tile
            else if CC_POSITIONS.contains(&current_pos) {
                // This line goes above `state.cc_chance_effects()`
                // since that modifies state.r#type.probability()
                let atp_probability = s.r#type.probability() / 21.;

                // Effects of rolling to a chance card tile
                let mut chance_effects = s.cc_chance_effects();
                for state in &mut chance_effects {
                    if state.lvl1rent_cc > 0 {
                        state.lvl1rent_cc -= 1
                    }
                }

                children.splice(children.len().., chance_effects);

                // Chance card: Move all players not in jail to free
                // parking. This is implemented here (instead of in
                // `cc_chance_effects()`) for optimisation purposes.
                if rolled_doubles {
                    atp_doubles_probability += atp_probability;
                } else {
                    atp_singles_probability += atp_probability;
                }

                match s.r#type {
                    // No need to push `state` to children if its probability is 0
                    StateType::Chance(p) if p == 0. => return,
                    _ => (),
                }
            } else if CORNER_POSITIONS.contains(&current_pos) {
                // This tile does nothing so it's the next player's turn
                s.setup_next_player();
            } else {
                // The player has to do something according to the tile they're on
                s.next_move_is_chance = false;
            }

            // The previous round has passed
            if s.lvl1rent_cc > 0 {
                s.lvl1rent_cc -= 1
            }

            // Store the new game state
            children.push(Box::new(s));
        };

        // Get the player out of jail if they're in jail
        if self.players[self.current_player_index].in_jail {
            // Try rolling doubles to get out of jail
            let double_probabilities = State::roll_for_doubles(3);

            // Loop through all possible dice results
            for roll in double_probabilities {
                // Derive a new game state from the current game state
                let mut new_state = State {
                    r#type: StateType::Chance(roll.probability),
                    ..self.clone()
                };

                // We didn't manage to roll doubles
                if !roll.is_double {
                    // $100 penalty for not rolling doubles
                    new_state.current_player().balance -= 100;
                }

                // Update the current player's position
                new_state.move_by(roll.sum);

                // `false` because rolling doubles to get out of
                // jail doesn't count towards your consecutive doubles
                push_state(new_state, false);
            }
        }
        // Otherwise, play as normal
        else {
            // Loop through all possible dice results
            for roll in &*SIGNIFICANT_ROLLS {
                // Derive a new game state from the current game state
                let mut new_state = State {
                    r#type: StateType::Chance(roll.probability),
                    ..self.clone()
                };

                // Update the current player's position
                new_state.move_by(roll.sum);

                // Check if the player landed on 'go to jail'
                if new_state.current_player().position == 27 {
                    new_state.send_current_to_jail();
                }
                // Check if this roll got doubles
                else if roll.is_double {
                    // Increment the doubles_rolled counter
                    new_state.current_player().doubles_rolled += 1;

                    // Go to jail after three consecutive doubles
                    if new_state.current_player().doubles_rolled == 3 {
                        new_state.send_current_to_jail();
                    }
                } else {
                    // Reset the doubles counter
                    new_state.current_player().doubles_rolled = 0;
                }

                push_state(new_state, roll.is_double);
            }
        }

        // Chance card: Move all players not in jail to free parking.
        // This is implemented here (instead of in `cc_chance_effects()`)
        // for optimisation purposes.

        // Set up "all to parking"'s single-roll state
        let mut atp_singles = State {
            r#type: StateType::Chance(atp_singles_probability),
            ..self.clone()
        };
        atp_singles.current_player().doubles_rolled = 0;

        // Set up "all to parking"'s double-roll state
        let mut atp_doubles = State {
            r#type: StateType::Chance(atp_doubles_probability),
            ..self.clone()
        };
        // Note: we know this won't increment to 3 because that logic is already implemented above
        atp_doubles.current_player().doubles_rolled += 1;

        // Move atp players to 'free parking' and update their `lvl1rent_cc`
        for mut atp in [atp_singles, atp_doubles] {
            for player in &mut atp.players {
                if !player.in_jail {
                    player.position = 18;
                }
            }

            if atp.lvl1rent_cc > 0 {
                atp.lvl1rent_cc -= 1
            }

            atp.setup_next_player();
            children.push(Box::new(atp));
        }

        children
    }
}

pub fn print_states(states: &Vec<Box<State>>) {
    for child in states {
        println!("{}", child);
    }

    println!("{} total child states", states.len());

    let total_probability: f64 = states
        .iter()
        .map(|s| match s.r#type {
            StateType::Chance(p) => p,
            StateType::Choice => 0.,
        })
        .sum();

    if total_probability != 0. {
        println!("Total probability: {}", total_probability);
    } else {
        println!("No total probability")
    }
}
