use std::time::Instant;
mod game;
use game::Game;

fn main() {
    let start = Instant::now();

    let mut game = Game::new(2);
    game.play();

    // let children = State::origin(2).children()[10].children();
    // println!("{}", State::origin(2).minimax(5).1);

    let duration = start.elapsed();

    // print_states(&children);
    println!("Time elapsed: {:?}", duration);
}
