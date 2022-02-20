use std::time::Instant;

mod game;
use game::{Agent, Game};

fn main() {
    let start = Instant::now();

    let mut game = Game::new(vec![Agent::new_ai(1000, 2., 0), Agent::new_human()]);
    game.play();

    let duration = start.elapsed();

    println!("Time elapsed: {:?}", duration);
}
