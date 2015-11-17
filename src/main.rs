extern crate tak;
extern crate iron;

use std::str::FromStr;
use std::env;
use iron::prelude::*;
use iron::status;

/// Look up our server port number in PORT, for compatibility with Heroku.
fn get_server_port() -> u16 {
    let port_str = env::var("PORT").unwrap_or(String::new());
    FromStr::from_str(&port_str).unwrap_or(8080)
}

fn main() {
    Iron::new(|req: &mut Request| -> IronResult<Response> {
        println!("{:?}", &req.url.path);
        let mut game = tak::Board::new(5);
        for turn in &req.url.path {
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
    }).http(("0.0.0.0", get_server_port())).unwrap();
    println!("On 3000");
}
