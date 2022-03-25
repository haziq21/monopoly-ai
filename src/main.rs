use std::time::Instant;

mod game;
use game::{Agent, Game};

fn main() {
    let start = Instant::now();

    Game::play(vec![Agent::new_ai(1000, 2., 0), Agent::new_random()]);

    let duration = start.elapsed();

    println!("Time elapsed: {:?}", duration);
}
