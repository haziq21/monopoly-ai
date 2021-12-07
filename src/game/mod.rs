use rand::random;

mod globals;
use globals::*;

mod state;
use state::State;

pub struct Game {
    pub player_weights: Vec<[f64; NUM_FACTORS]>,
    pub current_state: state::State,
}

impl Game {
    /*********        OUTWARD-FACING INTERFACES        *********/

    /// Create a new game with `player_count` players.
    pub fn new(player_count: usize) -> Game {
        let mut player_weights = Vec::with_capacity(player_count);

        for _ in 0..player_count {
            player_weights.push(random());
        }

        Game {
            player_weights,
            current_state: State::new(player_count),
        }
    }

    /// Play the game until it ends.
    pub fn play(&mut self) {
        // Placeholder
        self.minimax(&self.current_state, 1);
    }

    /*********        ALIASES        *********/

    fn player_count(&self) -> usize {
        self.player_weights.len()
    }

    /*********        HELPER FUNCTIONS        *********/

    /// Return the index of the player who would most likely "back out" of an auction the last.
    fn auction_winner(&self, state: &State, property_pos: u8) -> usize {
        fn to_nearest_20(x: u16) -> u16 {
            (x + 10) / 20 * 20
        }

        // The greatest prices each player would pay for
        let mut bail_prices = vec![0; self.player_count()];

        // Loop through all the players
        for p in 0..state.players.len() {
            // TODO: Account for property worth
            // TODO: Account for when max_price < 20
            let mut upper_bound = state.players[p].balance;
            let mut lower_bound = 20;
            let mut target_price = to_nearest_20((20 + upper_bound) / 2);

            // Binary search the property price
            loop {
                let p_weights = self.player_weights[p];

                // The evaluation where player `p` wins the auction
                let win_eval = {
                    let opp_balance = state.players.iter().map(|&p| p.balance).sum::<u16>()
                        - state.players[p].balance;
                    let p_balance = state.players[p].balance - target_price;
                    let eval_balance = (p_balance / opp_balance) as f64 * p_weights[0];

                    eval_balance
                };
                // The evaluation where player `p` doesn't win the auction
                let lose_eval = {
                    let opp_balance = state.players.iter().map(|&p| p.balance).sum::<u16>()
                        - state.players[p].balance
                        - target_price;
                    let p_balance = state.players[p].balance;
                    let eval_balance = (p_balance / opp_balance) as f64 * p_weights[0];

                    eval_balance
                };

                if win_eval > lose_eval {
                    // Narrow the range updwards and adjust the target price
                    lower_bound = target_price;
                    target_price = to_nearest_20((lower_bound + upper_bound) / 2);
                } else if lose_eval > win_eval {
                    // Narrow the range downwards and adjust the target price
                    upper_bound = target_price;
                    target_price = to_nearest_20((lower_bound + upper_bound) / 2);

                    // We have converged
                    if target_price == upper_bound {
                        bail_prices[p] = target_price - 20;
                        break;
                    }
                } else {
                    panic!("lose_eval == win_eval"); // Reachable, but I'll deal with it later
                }
            }
        }

        let winner = bail_prices
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(index, _)| index);

        winner.unwrap()
    }

    /*********        MINIMAX        *********/

    /// Return the static evaluation for one player.
    fn single_static_eval(&self, player_index: usize, state: &State) -> f64 {
        // let total_balance = self.players.iter().map(|&p| p.balance).sum();

        // The ratio of a player’s balance to the sum of the opponents’ balance
        state.players[player_index].balance as f64
    }

    /// Return the static evaluation of this state. Each evaluation
    /// is a linear combination of six factors:
    ///
    /// 1. The ratio of a player’s balance to the sum of the opponents’ balance.
    /// 2. The ratio of a player’s net property worth to the sum of the opponents’ net property worth.
    /// 3. The ratio of the sum of the rents of a player’s properties to the sum of the rents of the opponents’ properties.
    /// 4. Plys to go until the event card “all players pay rent for two rounds” wears off
    /// 5. Number of possible event cards that could come up next
    /// 6. TODO
    fn static_eval(&self, state: &State) -> Vec<f64> {
        let mut eval = Vec::with_capacity(self.player_weights.len());

        for i in 0..self.player_weights.len() {
            eval.push(self.single_static_eval(i, state))
        }

        eval
    }

    fn minimax(&self, state: &State, depth: u64) -> (Vec<f64>, u128) {
        if depth == 0 {
            return (self.static_eval(state), 0);
        }

        let mut best_eval = vec![0.; self.player_count()];
        let children = state.get_children();
        let mut total_len = children.len() as u128;

        for child in &children {
            let (eval, len) = self.minimax(child, depth - 1);
            total_len += len;
            if best_eval[state.current_player_index] < eval[state.current_player_index] {
                best_eval = eval;
            }
        }

        (best_eval, total_len)
    }
}
