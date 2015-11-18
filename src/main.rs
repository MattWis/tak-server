extern crate tak;
extern crate iron;
extern crate persistent;

use std::str::FromStr;
use std::env;
use std::collections::HashMap;
use iron::prelude::*;
use iron::status;
use persistent::Write;
use iron::typemap::Key;

/// Look up our server port number in PORT, for compatibility with Heroku.
fn get_server_port() -> u16 {
    let port_str = env::var("PORT").unwrap_or(String::new());
    FromStr::from_str(&port_str).unwrap_or(8080)
}

#[derive(Copy, Clone)]
pub struct Games;

impl Key for Games { type Value = HashMap<String, tak::Board>; }

fn serve_game(req: &mut Request) -> IronResult<Response> {
    println!("{:?}", &req.url.path);
    let mutex = req.get::<Write<Games>>().unwrap();
    let mut map = mutex.lock().unwrap();
    let key = &req.url.path[0];

    if !map.contains_key(key) {
        map.insert((*key).clone(), tak::Board::new(5));
    }
    let mut game = map.get_mut(key).unwrap();

    for turn in &req.url.path[1..] {
        let formatted = format!("Illegal move: {}", turn);
        let error = Response::with((status::Ok, formatted));
        match turn.parse::<tak::Turn>() {
            Ok(turn) => match tak::play(&turn, &mut game) {
                Ok(_) => (),
                Err(_) => return Ok(error),
            },
            Err(_) => return Ok(error),
        }
    }
    let response = match game.check_winner() {
        Some(tak::Player::One) => format!("{}\n Player 1 Wins!", &game),
        Some(tak::Player::Two) => format!("{}\n Player 2 Wins!", &game),
        None => format!("{}", &game),
    };

    Ok(Response::with((status::Ok, response)))
}


fn main() {
    let mut chain = Chain::new(serve_game);
    chain.link(Write::<Games>::both(HashMap::new()));
    Iron::new(chain).http(("0.0.0.0", get_server_port())).unwrap();
    println!("On 3000");
}
