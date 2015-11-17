extern crate tak;
extern crate iron;


use iron::prelude::*;
use iron::status;

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
    }).http("localhost:3000").unwrap();
    println!("On 3000");
}
