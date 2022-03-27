use rand::Rng;
use std::collections::HashMap;

mod globals;
use globals::*;

mod agent;
pub use agent::Agent;

mod state_diff;
use state_diff::{BranchType, DiffMessage, FieldDiff, MoveType, PropertyOwnership, StateDiff};

/// A simulation of Monopoly.
pub struct Game {
    /// Number of players playing the game.
    player_count: usize,
    /// The moves taken by players in terms of the indexes of the children.
    move_history: Vec<usize>,
    /// The current game state, as well as all its decendants.
    nodes: Vec<StateDiff>,
    /// Indexes of states that have been marked for deletion.
    /// These states can be safely replaced by newer states.
    dirty_handles: Vec<usize>,
    /// The index of the state the game is currently at.
    root_handle: usize,
}

impl Game {
    /*********       PUBLIC INTERFACES        *********/

    /// Return a new game.
    pub fn new(player_count: usize) -> Self {
        Self {
            player_count,
            move_history: vec![],
            nodes: vec![StateDiff::new_root(player_count)],
            dirty_handles: vec![],
            root_handle: 0,
        }
    }

    /// Play the game until it ends.
    pub fn play(mut agents: Vec<Agent>) {
        let mut game = Game::new(agents.len());

        while !game.is_terminal(game.root_handle) {
            game.gen_children_save(game.root_handle);

            let first_child = game.nodes[game.root_handle].children[0];
            let next_branch_type = game.diff_branch_type(first_child);
            let current_pindex = game.diff_current_pindex(game.root_handle);

            let next_node = match next_branch_type {
                BranchType::Chance(_) => game.get_any_chance_child(game.root_handle),
                BranchType::Choice => agents[current_pindex].make_choice(&mut game),
            };

            let pindex = game.diff_current_pindex(game.root_handle);

            game.advance_root_node(next_node);

            println!(
                "move hist: p{} -> {} ({})",
                pindex,
                game.move_history.last().unwrap(),
                game.nodes[game.root_handle].message
            );
        }

        println!("loser: {}", game.get_loser(game.root_handle));
    }

    /*********        HELPERS        *********/

    /// Push the new state node to `self.state_nodes` and return its handle.
    fn append_state(&mut self, state: StateDiff) -> usize {
        let i;
        let parent = state.parent;

        match self.dirty_handles.pop() {
            Some(handle) => {
                i = handle;
                self.nodes[i] = state;
            }
            None => {
                self.nodes.push(state);
                i = self.nodes.len() - 1;
            }
        }

        // Update parent state's children vector
        self.nodes[parent].children.push(i);

        i
    }

    /// Generate and append children.
    fn gen_children_save(&mut self, handle: usize) {
        if self.nodes[handle].children.len() == 0 && !self.is_terminal(handle) {
            for child in self.gen_children(handle) {
                self.append_state(child);
            }
        }
    }

    /// Set the root state to be one of the existing root state's children.
    /// `child_index` is not a regular handle, but the index of the target
    /// state in the current root node's `children` vec.
    fn advance_root_node(&mut self, child_index: usize) {
        let new_handle = self.nodes[self.root_handle]
            .children
            .swap_remove(child_index);

        // Mark the old handle and all of the new handle's siblings as 'dirty'
        self.dirty_handles.push(self.root_handle);
        for h in self.nodes[self.root_handle].children.clone() {
            self.mark_dirty(h);
        }

        // Ensure the new root node has every diff
        for d in DiffID::all() {
            if !self.nodes[new_handle].diff_exists(d) {
                let diff = self.diff_field(new_handle, d).clone();
                self.nodes[new_handle].set_diff(d, diff);
            }
        }

        // Update the game's move history
        self.move_history.push(child_index);

        // Set itself as its parent to ensure that there are
        // no more references to deleted nodes (just in case)
        self.nodes[new_handle].parent = new_handle;

        // Update the root handle
        self.root_handle = new_handle;
    }

    /// Mark a state and all of its descendants as 'dirty'.
    fn mark_dirty(&mut self, handle: usize) {
        self.dirty_handles.push(handle);

        // Mark all the descendants as 'dirty'
        for h in self.nodes[handle].children.clone() {
            self.mark_dirty(h);
        }
    }

    /// Return the player whose turn it currently is at the specified state.
    fn get_current_player(&self, handle: usize) -> &Player {
        &self.diff_players(handle)[self.diff_current_pindex(handle)]
    }

    /// Return the index of the player whose turn it will be next.
    fn get_next_pindex(&self, handle: usize) -> usize {
        (self.diff_current_pindex(handle) + 1) % self.player_count
    }

    /// Return the next value of `top_cc`.
    fn get_next_top_cc(&self, handle: usize) -> usize {
        (self.diff_top_cc(handle) + 1) % TOTAL_CHANCE_CARDS
    }

    /// Return the probabilities of all the child nodes of `handle`.
    /// This will return an empty vector if the `handle` node doesn't
    /// have any children. Panics if a child is not a chance node.
    fn get_children_chances(&self, handle: usize) -> Vec<f64> {
        let mut chances = vec![];

        for child_handle in &self.nodes[handle].children {
            match self.diff_branch_type(*child_handle) {
                BranchType::Chance(p) => chances.push(*p),
                _ => panic!("Choice node found in get_children_chances()"),
            }
        }

        chances
    }

    /// Return the index of a randomly selected child chance node.
    /// Note that this returns the node's index in `handle`'s `children`
    /// vector, not a handle that can used in `game.nodes[handle]`.
    fn get_any_chance_child(&self, handle: usize) -> usize {
        let chances = self.get_children_chances(handle);
        let mut rng = rand::thread_rng();
        let mut pos: f64 = rng.gen();

        for (i, &c) in chances.iter().enumerate() {
            if pos <= c {
                return i;
            }

            pos -= c;
        }

        // Just in case of floating-point arithmetic inacuraccies
        chances.len() - 1
    }

    /// Return a `StateDiff` with the boilerplate for chance cards:
    /// - Sets `next_move` to `Roll`
    /// - Updates `current_player` if needed
    /// - Updates `seen_ccs` or `top_cc`
    fn new_state_from_cc(&self, card: ChanceCard, handle: usize) -> StateDiff {
        let mut state = StateDiff::new_with_parent(handle);
        state.next_move = MoveType::Roll;

        // It's the next player's turn if the current player didn't roll doubles
        if self.get_current_player(handle).doubles_rolled == 0 {
            state.set_current_pindex(self.get_next_pindex(handle));
        }

        // Update the top_cc if needed
        if self.diff_seen_ccs(handle).len() == TOTAL_CHANCE_CARDS {
            state.set_top_cc(self.get_next_top_cc(handle));
        } else {
            let mut seen_ccs = self.diff_seen_ccs(handle).clone();
            seen_ccs.push(card);
            state.set_seen_ccs(seen_ccs);
        }

        state
    }

    /// Modify the state to be the next player's turn if the current player didn't roll doubles.
    /// This only affects the state's next_move and current_pindex
    fn advance_move(&self, handle: usize, state: &mut StateDiff) {
        state.next_move = MoveType::Roll;

        if self.get_current_player(handle).doubles_rolled == 0 {
            state.set_current_pindex(self.get_next_pindex(handle));
        }
    }

    fn get_auction_winner_chances(&self, handle: usize) -> Vec<(usize, f64)> {
        let possible_winners = self
            .diff_players(handle)
            .iter()
            .enumerate()
            .filter(|(_, p)| p.balance > 20);
        let total_balance = possible_winners
            .clone()
            .map(|(_, p)| p.balance)
            .sum::<i32>() as f64;

        possible_winners
            .map(|(i, p)| (i, p.balance as f64 / total_balance))
            .collect()
    }

    fn get_winning_bid_chances(&self, handle: usize, winner: usize) -> Vec<(i32, f64)> {
        let balance = self.diff_players(handle)[winner].balance;
        let balance_at_pos =
            |pos: f64| ((balance - 20) as f64 * pos / 20.0).round() as i32 * 20 + 20;

        if balance <= 20 {
            // Just in case...
            panic!("get_winning_bid_chances() received players with <= $20");
        }

        // Based on a bell curve
        [
            (1. / 6., 0.0675),
            (2. / 6., 0.2410),
            (3. / 6., 0.3830),
            (4. / 6., 0.2410),
            (5. / 6., 0.0675),
        ]
        .iter()
        .map(|(pos, chance)| (balance_at_pos(*pos), *chance))
        .fold(vec![], |mut acc, (p, c)| {
            if let Some(last) = acc.last_mut() {
                if p == last.0 {
                    last.1 += c;
                    return acc;
                }
            }

            acc.push((p, c));
            acc
        })
    }

    fn is_terminal(&self, handle: usize) -> bool {
        self.diff_players(handle).iter().any(|p| p.balance < 0)
    }

    fn get_loser(&self, handle: usize) -> usize {
        if !self.is_terminal(handle) {
            panic!("non-terminal state found while getting loser");
        }

        let losers: Vec<usize> = self
            .diff_players(handle)
            .iter()
            .enumerate()
            .filter(|(_, p)| p.balance <= 0)
            .map(|(i, _)| i)
            .collect();
        if losers.len() > 1 {
            panic!("more than 1 loser");
        }

        losers[0]
    }

    // fn get_gameover_info(&self, handle: usize) {}

    /*********        STATE DIFF GETTERS        *********/

    fn diff_field(&self, handle: usize, diff_id: DiffID) -> &FieldDiff {
        // Alias for the state
        let s = &self.nodes[handle];

        match s.get_diff_index(diff_id) {
            Some(i) => &s.diffs[i],
            None => self.diff_field(s.parent, diff_id),
        }
    }

    /// Return the branch type of the state.
    fn diff_branch_type(&self, handle: usize) -> &BranchType {
        match self.diff_field(handle, DiffID::BranchType) {
            FieldDiff::BranchType(x) => x,
            _ => unreachable!(),
        }
    }

    /// Return a vector of players playing the game at the specified state.
    fn diff_players(&self, handle: usize) -> &Vec<Player> {
        match self.diff_field(handle, DiffID::Players) {
            FieldDiff::Players(x) => x,
            _ => unreachable!(),
        }
    }

    /// Return the index of the player whose turn it currently is at the specified state.
    fn diff_current_pindex(&self, handle: usize) -> usize {
        match self.diff_field(handle, DiffID::CurrentPlayer) {
            FieldDiff::CurrentPlayer(x) => *x,
            _ => unreachable!(),
        }
    }

    /// Return the properties that are owned by players at the specified state.
    fn diff_owned_properties(&self, handle: usize) -> &HashMap<u8, PropertyOwnership> {
        match self.diff_field(handle, DiffID::OwnedProperties) {
            FieldDiff::OwnedProperties(x) => x,
            _ => unreachable!(),
        }
    }

    /// Return a vector of chance cards that have already been seen from the specified state.
    fn diff_seen_ccs(&self, handle: usize) -> &Vec<ChanceCard> {
        match self.diff_field(handle, DiffID::SeenCcs) {
            FieldDiff::SeenCCs(x) => x,
            _ => unreachable!(),
        }
    }

    /// Return top_cc from the specified state.
    fn diff_top_cc(&self, handle: usize) -> usize {
        match self.diff_field(handle, DiffID::SeenCcsHead) {
            FieldDiff::SeenCCsHead(x) => *x,
            _ => unreachable!(),
        }
    }

    /// Return the specified state's `Level1Rent`.
    fn diff_lvl_1_rent(&self, handle: usize) -> u8 {
        match self.diff_field(handle, DiffID::Level1Rent) {
            FieldDiff::Level1Rent(x) => *x,
            _ => unreachable!(),
        }
    }

    /*********        GENERAL STATE GENERATION        *********/

    /// Return child states that can be reached from the specified state.
    fn gen_children(&self, handle: usize) -> Vec<StateDiff> {
        let mut children = match self.nodes[handle].next_move {
            MoveType::Roll => self.gen_roll_children(handle),
            MoveType::ChanceCard => self.gen_cc_children(handle),
            MoveType::ChoicefulCC(cc) => self.gen_choiceful_cc_children(handle, cc),
            MoveType::Property => self.gen_property_children(handle),
            MoveType::SellProperty => self.gen_sell_prop_children(handle),
            MoveType::Auction => self.gen_auction_children(handle),
            MoveType::Location => self.gen_location_children(handle),
            MoveType::Undefined => unreachable!(),
        };

        let lvl_1_rent = self.diff_lvl_1_rent(handle);
        if lvl_1_rent > 0 {
            for child in &mut children {
                // Check if it's the next player's turn
                if child.diff_exists(DiffID::CurrentPlayer) {
                    child.set_level_1_rent(lvl_1_rent - 1);
                }
            }
        }

        children
    }

    /// Return child states that can be reached by rolling dice from the specified state.
    fn gen_roll_children(&self, handle: usize) -> Vec<StateDiff> {
        // The index of the player whose turn it currently is
        let i = self.diff_current_pindex(handle);
        let mut children = vec![];

        // Get the player out of jail if they're in jail
        if self.get_current_player(handle).in_jail {
            // Try rolling doubles to get out of jail
            let double_probabilities = roll_for_doubles(3);

            // Loop through all possible dice results
            for roll in double_probabilities {
                // Create a new diff
                let mut diff = StateDiff::new_with_parent(handle);
                // Update the branch type
                diff.set_branch_type(BranchType::Chance(roll.probability));
                // Clone the players
                let mut players = self.diff_players(handle).clone();
                // Update the current player's position
                players[i].move_by(roll.sum);

                // We didn't manage to roll doubles
                if !roll.is_double {
                    // $100 penalty for not rolling doubles
                    players[i].balance -= 100;
                }

                diff.message = DiffMessage::Roll(players[i].position);
                // Set the next move
                diff.next_move = MoveType::when_landed_on(players[i].position);
                // Set the players diff
                diff.set_players(players);
                // Update the current_player if needed
                if diff.next_move.is_roll() {
                    diff.set_current_pindex(self.get_next_pindex(handle));
                }

                children.push(diff);
            }
        }
        // Otherwise, play as normal
        else {
            // Loop through all possible dice results
            for roll in SIGNIFICANT_ROLLS.iter() {
                // Create a new state
                let mut state = StateDiff::new_with_parent(handle);
                // Update the branch type
                state.set_branch_type(BranchType::Chance(roll.probability));
                // Clone the players
                let mut players = self.diff_players(handle).clone();
                // Update the current player's position
                players[i].move_by(roll.sum);

                // Check if the player landed on 'go to jail'
                if players[i].position == GO_TO_JAIL_POSITION {
                    players[i].send_to_jail();
                    state.message = DiffMessage::RollToJail;
                }
                // Check if this roll got doubles
                else if roll.is_double {
                    // Increment the doubles_rolled counter
                    players[i].doubles_rolled += 1;

                    // Go to jail after three consecutive doubles
                    if players[i].doubles_rolled == 3 {
                        players[i].send_to_jail();
                        state.message = DiffMessage::RollToJail;
                    } else {
                        state.message = DiffMessage::RollDoubles(players[i].position);
                    }
                } else {
                    // Reset the doubles counter
                    players[i].doubles_rolled = 0;
                    state.message = DiffMessage::Roll(players[i].position);
                }

                // Set the next move
                state.next_move = MoveType::when_landed_on(players[i].position);
                // Update the current_player if needed
                if state.next_move.is_roll() && players[i].doubles_rolled == 0 {
                    state.set_current_pindex(self.get_next_pindex(handle));
                }
                // Set the players diff
                state.set_players(players);

                children.push(state);
            }
        }

        children
    }

    /// Return child states that can be reached by picking a chance card from the specified state.
    fn gen_cc_children(&self, handle: usize) -> Vec<StateDiff> {
        let mut children = vec![];
        let seen_ccs = self.diff_seen_ccs(handle);

        // We can deduce the exact chance card that we're going to get since we've seen them all
        if seen_ccs.len() == TOTAL_CHANCE_CARDS {
            // The chance card that the player will definitely get
            let definite_cc = seen_ccs[self.diff_top_cc(handle)];

            // Get the child diffs according to the choicefulness of the chance card
            if definite_cc.is_choiceless() {
                // This is the only possibility since this is a choiceless chance card
                return vec![self.gen_choiceless_cc_child(definite_cc, handle, 1.)];
            }

            return self.gen_choiceful_cc_children(handle, definite_cc);
        }

        // We can't know the exact chance card that we're
        // going to get, so calculate all their probabilities
        let unseen_cards = ChanceCard::unseen_counts(&seen_ccs);

        for (card, count) in unseen_cards {
            // Skip if the chance card has no chance of occurring
            if count == 0 {
                continue;
            }

            // Calculate the probability of encountering this chance card
            let probability = count as f64 / (TOTAL_CHANCE_CARDS - seen_ccs.len()) as f64;

            if card.is_choiceless() {
                children.push(self.gen_choiceless_cc_child(card, handle, probability));
            } else {
                let mut state = StateDiff::new_with_parent(handle);
                state.message = DiffMessage::ChanceCard(card);
                state.set_branch_type(BranchType::Chance(probability));
                state.next_move = MoveType::ChoicefulCC(card);
                children.push(state);
            };
        }

        children
    }

    /// Return child states that can be reached by landing on a location tile.
    fn gen_location_children(&self, handle: usize) -> Vec<StateDiff> {
        let mut children = vec![];
        let curr_pindex = self.diff_current_pindex(handle);

        for &pos in PROP_POSITIONS.iter() {
            let mut players = self.diff_players(handle).clone();

            // Pay $100
            players[curr_pindex].balance -= 100;
            // Move to a property
            players[curr_pindex].position = pos;

            // Add the new state to children
            let mut new_state = StateDiff::new_with_parent(handle);
            new_state.message = DiffMessage::Location(pos);
            new_state.next_move = MoveType::Property;
            new_state.set_branch_type(BranchType::Choice);
            new_state.set_players(players);
            children.push(new_state);
        }

        // There's also the option to do nothing
        let mut no_move = StateDiff::new_with_parent(handle);
        no_move.message = DiffMessage::NoLocation;
        self.advance_move(handle, &mut no_move);
        no_move.set_branch_type(BranchType::Choice);
        children.push(no_move);

        children
    }

    /// Return child states that can be reached by landing on a property.
    /// This assumes that the current player is on a property tile.
    fn gen_property_children(&self, handle: usize) -> Vec<StateDiff> {
        let player_pos = self.get_current_player(handle).position;
        let curr_pindex = self.diff_current_pindex(handle);
        let mut children = vec![];

        // Check if the property at the player's location is owned
        if let Some(prop) = self.diff_owned_properties(handle).get(&player_pos) {
            let mut new_state = StateDiff::new_with_parent(handle);
            new_state.set_branch_type(BranchType::Chance(1.));

            // The current player owes rent to the owner of this property
            if prop.owner != curr_pindex {
                // TODO: Implement SellProp MoveType
                let mut players = self.diff_players(handle).clone();
                let new_rent_level = if self.diff_lvl_1_rent(handle) == 0 {
                    prop.rent_level
                } else {
                    1
                };
                let balance_due = PROPERTIES[&player_pos].rents[new_rent_level - 1];

                // Pay the owner using the current player's money
                players[curr_pindex].balance -= balance_due;
                players[prop.owner].balance += balance_due;
                new_state.set_players(players);

                new_state.message = DiffMessage::LandOppProp;
            } else {
                new_state.message = DiffMessage::LandOwnProp;
            }

            // Raise the rent level
            let mut props = self.diff_owned_properties(handle).clone();
            props.get_mut(&player_pos).unwrap().raise_rent();
            new_state.set_owned_properties(props);

            // It's the next turn now
            self.advance_move(handle, &mut new_state);

            return vec![new_state];
        } // At this point, the property isn't owned, so the player has to decide whether to buy or auction

        let curr_player_balance = self.diff_players(handle)[curr_pindex].balance;
        // Check if the player has enough money to buy the property
        if curr_player_balance > PROPERTIES[&player_pos].price {
            // The state where the player buys the property
            let mut buy_state = StateDiff::new_with_parent(handle);
            buy_state.message = DiffMessage::BuyProp;
            self.advance_move(handle, &mut buy_state);
            buy_state.set_branch_type(BranchType::Choice);
            // New players
            let mut buy_state_players = self.diff_players(handle).clone();
            buy_state_players[curr_pindex].balance -= PROPERTIES[&player_pos].price;
            buy_state.set_players(buy_state_players);
            // New owned properties
            let mut buy_state_props = self.diff_owned_properties(handle).clone();
            buy_state_props.insert(
                player_pos,
                PropertyOwnership {
                    owner: curr_pindex,
                    rent_level: 1,
                },
            );
            buy_state.set_owned_properties(buy_state_props);

            children.push(buy_state);
        }

        // The state where the player auctions the property
        let mut auction_state = StateDiff::new_with_parent(handle);
        auction_state.message = DiffMessage::AuctionProp;
        auction_state.set_branch_type(BranchType::Choice);
        auction_state.next_move = MoveType::Auction;
        children.push(auction_state);

        children
    }

    /// Return child states that can be reached by auctioning a property.
    /// This assumes that the current player is on a property tile.
    fn gen_auction_children(&self, handle: usize) -> Vec<StateDiff> {
        let mut children = vec![];

        // Loop through all the possible auction winners and winning bids
        for (auction_winner, player_chance) in self.get_auction_winner_chances(handle) {
            for (winning_bid, bid_chance) in self.get_winning_bid_chances(handle, auction_winner) {
                let mut players = self.diff_players(handle).clone();
                let mut props = self.diff_owned_properties(handle).clone();
                let mut new_state = StateDiff::new_with_parent(handle);
                new_state.message =
                    DiffMessage::AfterAuction(auction_winner, winning_bid, player_chance);

                // It's the current player who is on the property that is being auctioned,
                // so we use their position instead of the position of the player who won the auction
                let prop_pos = players[self.diff_current_pindex(handle)].position;

                // The auction winner pays the bid...
                players[auction_winner].balance -= winning_bid;
                // ...to get the property
                props.insert(
                    prop_pos,
                    PropertyOwnership {
                        owner: auction_winner,
                        rent_level: 1,
                    },
                );

                new_state.set_players(players);
                new_state.set_owned_properties(props);
                new_state.set_branch_type(BranchType::Chance(player_chance * bid_chance));

                self.advance_move(handle, &mut new_state);
                children.push(new_state);
            }
        }

        if children.len() == 0 {
            let mut state = StateDiff::new_with_parent(handle);
            self.advance_move(handle, &mut state);
            children.push(state);
        }

        children
    }

    fn gen_sell_prop_children(&self, handle: usize) -> Vec<StateDiff> {
        let mut children = vec![];
        let curr_pindex = self.diff_current_pindex(handle);

        // TODO: Optimise this
        for (prop_pos, prop) in self.diff_owned_properties(handle) {
            if prop.owner != curr_pindex {
                continue;
            }

            let mut sell_prop = StateDiff::new_with_parent(handle);

            // Sell the property to the bank
            let mut props = self.diff_owned_properties(handle).clone();
            props.remove(&prop_pos);
            sell_prop.set_owned_properties(props);
            // The player gets the money
            let mut players = self.diff_players(handle).clone();
            players[curr_pindex].balance += PROPERTIES[&prop_pos].price;

            // If the player is still bankrupt, repeat the move
            if players[curr_pindex].balance < 0 {
                sell_prop.next_move = MoveType::SellProperty;
            } else {
                self.advance_move(handle, &mut sell_prop);
            }

            sell_prop.set_players(players);
            children.push(sell_prop);
        }

        if children.len() == 0 {
            // This state doesn't need a `next_move` because it's a terminal state
            vec![StateDiff::new_with_parent(handle)]
        } else {
            children
        }
    }

    /*********        CHOICEFUL CC STATE GENERATION        *********/

    /// Return child states that can be reached by getting a choiceful chance card.
    fn gen_choiceful_cc_children(&self, handle: usize, cc: ChanceCard) -> Vec<StateDiff> {
        let children = match cc {
            ChanceCard::RentTo5 => self.gen_cc_rent_to_x(true, handle),
            ChanceCard::RentTo1 => self.gen_cc_rent_to_x(false, handle),
            ChanceCard::SetRentInc => self.gen_cc_set_rent_change(true, handle),
            ChanceCard::SetRentDec => self.gen_cc_set_rent_change(false, handle),
            ChanceCard::SideRentInc => self.gen_cc_side_rent_change(true, handle),
            ChanceCard::SideRentDec => self.gen_cc_side_rent_change(false, handle),
            ChanceCard::RentSpike => self.gen_cc_rent_spike(handle),
            ChanceCard::Bonus => self.gen_cc_bonus(handle),
            ChanceCard::SwapProperty => self.gen_cc_swap_property(handle),
            ChanceCard::OpponentToJail => self.gen_cc_opponent_to_jail(handle),
            ChanceCard::GoToAnyProperty => self.gen_cc_go_to_any_property(handle),
            _ => panic!("choiceless cc passed to Game.gen_choiceful_cc_children()"),
        };

        if children.len() > 0 {
            children
        } else {
            vec![self.new_state_from_cc(cc, handle)]
        }
    }

    fn gen_cc_rent_to_x(&self, max: bool, handle: usize) -> Vec<StateDiff> {
        let mut children = vec![];
        let curr_pindex = self.diff_current_pindex(handle);
        let (cc, target_rent) = if max {
            (ChanceCard::RentTo5, 5)
        } else {
            (ChanceCard::RentTo1, 1)
        };

        for (pos, prop) in self.diff_owned_properties(handle) {
            // "RentTo5" only applies to your properties (not opponents), and we don't
            // need to add another child node if the rent level is already at its max/min
            if max && prop.owner != curr_pindex || prop.rent_level == target_rent {
                continue;
            }

            // Create the diff
            let mut child = self.new_state_from_cc(cc, handle);
            child.set_branch_type(BranchType::Choice);

            // Update the owned_properties
            let mut owned_props = self.diff_owned_properties(handle).clone();
            owned_props.get_mut(&pos).unwrap().rent_level = target_rent;
            child.set_owned_properties(owned_props);

            children.push(child);
        }

        children
    }

    fn gen_cc_set_rent_change(&self, increase: bool, handle: usize) -> Vec<StateDiff> {
        let mut children = vec![];
        let cc = if increase {
            ChanceCard::SetRentInc
        } else {
            ChanceCard::SetRentDec
        };

        // Loop through each color set
        for (_, positions) in PROPS_BY_COLOR.iter() {
            let mut owned_props = self.diff_owned_properties(handle).clone();
            let mut has_effect = false;

            // Loop through all the properties in this color set
            for pos in positions {
                // Check if a property exists at `pos`
                if let Some(prop) = owned_props.get_mut(&pos) {
                    has_effect |= prop.change_rent(increase);
                }
            }

            // Only store the new state if it's different
            if has_effect {
                let mut new_state = self.new_state_from_cc(cc, handle);
                new_state.set_branch_type(BranchType::Choice);
                new_state.set_owned_properties(owned_props);
                children.push(new_state);
            }
        }

        children
    }

    fn gen_cc_side_rent_change(&self, increase: bool, handle: usize) -> Vec<StateDiff> {
        let mut children = vec![];
        let cc = if increase {
            ChanceCard::SideRentInc
        } else {
            ChanceCard::SideRentDec
        };

        for positions in PROPS_BY_SIDE.iter() {
            let mut owned_properties = self.diff_owned_properties(handle).clone();
            let mut has_effect = false;

            for pos in positions {
                // Check if the property is owned
                if let Some(prop) = owned_properties.get_mut(&pos) {
                    has_effect |= prop.change_rent(increase);
                }
            }

            // Save the child if it's different
            if has_effect {
                let mut child = self.new_state_from_cc(cc, handle);
                child.set_branch_type(BranchType::Choice);
                child.set_owned_properties(owned_properties);
                children.push(child);
            }
        }

        children
    }

    fn gen_cc_rent_spike(&self, handle: usize) -> Vec<StateDiff> {
        let mut children = vec![];
        let i = self.diff_current_pindex(handle);

        for (pos, prop) in self.diff_owned_properties(handle) {
            // Skip if this property isn't owned by the current player
            if prop.owner != i {
                continue;
            }

            let mut properties = self.diff_owned_properties(handle).clone();
            let mut has_effect = false;

            // Raise this property's rent level
            has_effect |= properties.get_mut(&pos).unwrap().raise_rent();

            // Lower neighbours' rent levels (if they're owned)
            for n_pos in PROPERTY_NEIGHBOURS[&pos] {
                if let Some(n_prop) = properties.get_mut(&n_pos) {
                    has_effect |= n_prop.lower_rent();
                }
            }

            // Store new state if it's different
            if has_effect {
                let mut state = self.new_state_from_cc(ChanceCard::RentSpike, handle);
                state.set_branch_type(BranchType::Choice);
                state.set_owned_properties(properties);
                children.push(state);
            }
        }

        children
    }

    fn gen_cc_bonus(&self, handle: usize) -> Vec<StateDiff> {
        let mut children = vec![];
        let curr_pindex = self.diff_current_pindex(handle);

        for i in 0..self.player_count {
            // Skip the current player
            if i == curr_pindex {
                continue;
            }

            let mut players = self.diff_players(handle).clone();

            // Award $200 bonus to this player
            players[curr_pindex].balance += 200;

            // Award $200 bonus to an opponent
            players[i].balance += 200;

            // Add the new state
            let mut new_state = self.new_state_from_cc(ChanceCard::Bonus, handle);
            new_state.set_branch_type(BranchType::Choice);
            new_state.set_players(players);
            children.push(new_state);
        }

        children
    }

    fn gen_cc_swap_property(&self, handle: usize) -> Vec<StateDiff> {
        let mut children = vec![];
        let parent_props = self.diff_owned_properties(handle);
        let curr_pindex = self.diff_current_pindex(handle);

        // Loop through my properties
        for (my_pos, my_prop) in parent_props {
            // Skip opponent properties
            if my_prop.owner != curr_pindex {
                continue;
            }

            // Loop through opponent properties
            for (opp_pos, opp_prop) in parent_props {
                // Skip my properties
                if opp_prop.owner == curr_pindex {
                    continue;
                }

                // Swap properties
                let mut props = parent_props.clone();
                props.get_mut(&my_pos).unwrap().owner = opp_prop.owner;
                props.get_mut(&opp_pos).unwrap().owner = my_prop.owner;

                // Add the new state
                let mut new_state = self.new_state_from_cc(ChanceCard::SwapProperty, handle);
                new_state.set_branch_type(BranchType::Choice);
                new_state.set_owned_properties(props);
                children.push(new_state);
            }
        }

        children
    }

    fn gen_cc_opponent_to_jail(&self, handle: usize) -> Vec<StateDiff> {
        let mut children = vec![];
        let curr_pindex = self.diff_current_pindex(handle);

        for i in 0..self.player_count {
            // Skip the current player
            if i == curr_pindex {
                continue;
            }

            // Send the opponent to jail
            let mut players = self.diff_players(handle).clone();
            players[i].send_to_jail();

            // Add the new state
            let mut new_state = self.new_state_from_cc(ChanceCard::OpponentToJail, handle);
            new_state.set_branch_type(BranchType::Choice);
            new_state.set_players(players);
            children.push(new_state);
        }

        children
    }

    fn gen_cc_go_to_any_property(&self, handle: usize) -> Vec<StateDiff> {
        let mut children = vec![];
        let curr_pindex = self.diff_current_pindex(handle);

        for pos in PROP_POSITIONS.iter() {
            // Move the player to any property
            let mut players = self.diff_players(handle).clone();
            players[curr_pindex].position = *pos;

            // Create the new state
            let mut new_state = StateDiff::new_with_parent(handle);
            new_state.set_branch_type(BranchType::Choice);
            new_state.set_players(players);
            new_state.next_move = MoveType::Property;

            // Update top_cc or seen_ccs
            if self.diff_seen_ccs(handle).len() == TOTAL_CHANCE_CARDS {
                new_state.set_top_cc(self.get_next_top_cc(handle));
            } else {
                let mut seen_ccs = self.diff_seen_ccs(handle).clone();
                seen_ccs.push(ChanceCard::GoToAnyProperty);
                new_state.set_seen_ccs(seen_ccs);
            }

            children.push(new_state);
        }

        children
    }

    /*********        CHOICELESS CC STATE GENERATION        *********/

    /// Return child states that can be reached by getting a choiceless chance card.
    fn gen_choiceless_cc_child(
        &self,
        cc: ChanceCard,
        handle: usize,
        probability: f64,
    ) -> StateDiff {
        match cc {
            ChanceCard::PropertyTax => self.gen_cc_property_tax(probability, handle),
            ChanceCard::Level1Rent => self.gen_cc_level_1_rent(probability, handle),
            ChanceCard::AllToParking => self.gen_cc_all_to_parking(probability, handle),
            _ => panic!("choiceful cc passed to Game.gen_choiceless_cc()"),
        }
    }

    fn gen_cc_property_tax(&self, probability: f64, handle: usize) -> StateDiff {
        let mut tax = 0;
        let i = self.diff_current_pindex(handle);

        // Tax $50 per property owned
        for (_, prop) in self.diff_owned_properties(handle) {
            if prop.owner == i {
                tax += 50;
            }
        }

        // Clone the players
        let mut updated_players = self.diff_players(handle).clone();
        // Update the players based on the calculated tax
        updated_players[i].balance -= tax;

        // Create a new state
        let mut state = self.new_state_from_cc(ChanceCard::PropertyTax, handle);
        state.set_branch_type(BranchType::Chance(probability));
        state.set_players(updated_players);

        state
    }

    fn gen_cc_level_1_rent(&self, probability: f64, handle: usize) -> StateDiff {
        let mut state = self.new_state_from_cc(ChanceCard::Level1Rent, handle);
        state.set_branch_type(BranchType::Chance(probability));
        // Set the diff to 2 rounds (player_count * 2 turns per player)
        state.set_level_1_rent(self.player_count as u8 * 2);

        state
    }

    fn gen_cc_all_to_parking(&self, probability: f64, handle: usize) -> StateDiff {
        // Clone players
        let mut updated_players = self.diff_players(handle).clone();

        // Move every player who's not in jail to free parking
        for player in &mut updated_players {
            if !player.in_jail {
                player.position = FREE_PARKING_POSITION;
            }
        }

        // Create a new state
        let mut state = self.new_state_from_cc(ChanceCard::AllToParking, handle);
        state.set_branch_type(BranchType::Chance(probability));
        state.set_players(updated_players);

        state
    }
}
