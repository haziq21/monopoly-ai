use std::thread;

mod game;
use game::{Agent, Game};

fn main() {
    // 4 threads for multi-threading
    for _ in 0..4 {
        thread::spawn(|| loop {
            // Continuously run the simulations
            Game::play(vec![Agent::new_ai(2000, 2., 0), Agent::new_random()]);
        });
    }
}
