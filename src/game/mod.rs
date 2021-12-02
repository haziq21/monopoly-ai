use rand::distributions::{Distribution, Uniform};

mod globals;
use globals::*;

mod state;

pub struct Game {
    pub player_weights: Vec<[f64; NUM_FACTORS]>,
    pub current_state: state::State,
}

impl Game {
    pub fn new(player_count: usize) -> Game {
        let mut player_weights = Vec::with_capacity(player_count);

        for _ in 0..player_count {
            player_weights.push(rand::random());
        }

        Game {
            player_weights,
            current_state: state::State::origin(player_count),
        }
    }
}
