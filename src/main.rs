use std::time::Instant;
mod game;
use game::{Agent, Game};

fn main() {
    let start = Instant::now();

    let mut game = Game::new(vec![Agent::Ai, Agent::Human]);
    game.play();

    let duration = start.elapsed();

    println!("Time elapsed: {:?}", duration);
}
