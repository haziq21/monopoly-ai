use std::time::Instant;
mod game;
use game::Game;

fn main() {
    let start = Instant::now();

    let mut game = Game::new(2);
    game.insert_ai_agent();
    game.insert_human_agent();
    game.play();

    let duration = start.elapsed();

    println!("Time elapsed: {:?}", duration);
}
