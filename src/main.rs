extern crate tak;
extern crate iron;
extern crate persistent;
#[macro_use]
extern crate router;
extern crate params;
extern crate mount;
extern crate staticfile;
extern crate rustc_serialize;

use std::str::FromStr;
use std::env;
use std::collections::HashMap;
use std::path::Path;
use iron::prelude::*;
use iron::status;
use iron::mime::Mime;
use iron::typemap::Key;
use persistent::Write;
use mount::Mount;
use staticfile::Static;
use rustc_serialize::json;


fn wrap_html(body: String) -> String {
    format!("<!DOCTYPE html PUBLIC \"-//W3C//DTD HTML 4.01//EN\"\
            \"http://www.w3.org/TR/html4/strict.dt\">\n\
             <html lang=\"en\">\n<head>\n\
             <meta http-equiv=\"content-type\" content=\"text/html; charset=utf-8\">\n\
             <title>Tak Online</title>\n</head>\n\
             <body>\n<div>{}</div>\n</body>\n</html>", body).into()
}

fn respond_html(body: String) -> IronResult<Response> {
    let html = wrap_html(body);
    let content_type = "text/html".parse::<Mime>().unwrap();
    Ok(Response::with((content_type, status::Ok, html)))
}

fn respond_game(text: &tak::Game) -> IronResult<Response> {
    respond_html(format!("<body><pre>{}</pre><form method=\"POST\">
            <input type=\"text\" name=\"turn\"></input><br>
            <input type=\"submit\" value=\"Submit Move\"></input>
            </form> </body>", text))
}

/// Look up our server port number in PORT, for compatibility with Heroku.
fn get_server_port() -> u16 {
    let port_str = env::var("PORT").unwrap_or(String::new());
    FromStr::from_str(&port_str).unwrap_or(8080)
}

#[derive(Copy, Clone)]
pub struct Games;

pub struct Match {
    game: tak::Game,
    p1_claimed: bool,
    p2_claimed: bool,
}

impl Match {
    fn new(size: usize) -> Match {
        Match { game: tak::Game::new(size), p1_claimed: false, p2_claimed: false }
    }
}


impl Key for Games { type Value = HashMap<String, Match>; }

fn list_games(req: &mut Request) -> IronResult<Response> {
    let mutex = req.get::<Write<Games>>().unwrap();
    let map = mutex.lock().unwrap();
    let mut response: String = "Games Being Played<br><br>".into();
    for (key, _) in map.iter() {
        response.push_str(&(format!("{}<br>", key)));
    }
    respond_html(response)
}

fn serve_game(req: &mut Request) -> IronResult<Response> {
    let mutex = req.get::<Write<Games>>().unwrap();
    let key: String = req.extensions.get::<router::Router>().unwrap().find("gameId").unwrap().into();
    let mut map = mutex.lock().unwrap();
    if !map.contains_key(&key) {
        map.insert(key.clone(), Match::new(5));
    }
    let game = &map.get(&key).unwrap().game;

    respond_game(game)
}

fn game_json(req: &mut Request) -> IronResult<Response> {
    let mutex = req.get::<Write<Games>>().unwrap();
    let key: String = req.extensions.get::<router::Router>().unwrap().find("gameId").unwrap().into();
    let mut map = mutex.lock().unwrap();
    if !map.contains_key(&key) {
        map.insert(key.clone(), Match::new(5));
    }
    let game = &map.get(&key).unwrap().game;

    let content_type = "text/json".parse::<Mime>().unwrap();
    Ok(Response::with((content_type, status::Ok, json::encode(game).unwrap())))
}

fn claim_player(req: &mut Request) -> IronResult<Response> {
    let mutex = req.get::<Write<Games>>().unwrap();
    let key: String = req.extensions.get::<router::Router>().unwrap().find("gameId").unwrap().into();
    let mut map = mutex.lock().unwrap();
    if !map.contains_key(&key) {
        map.insert(key.clone(), Match::new(5));
    }
    let mut game = map.get_mut(&key).unwrap();

    let content_type = "text/json".parse::<Mime>().unwrap();
    if !game.p1_claimed {
        game.p1_claimed = true;
        Ok(Response::with((content_type, status::Ok, "{\"player\": 1}")))
    } else if !game.p2_claimed {
        game.p2_claimed = true;
        Ok(Response::with((content_type, status::Ok, "{\"player\": 2}")))
    } else {
        Ok(Response::with((content_type, status::Ok, "{\"player\": null}")))
    }

}

fn play_move(req: &mut Request) -> IronResult<Response> {
    let mutex = req.get::<Write<Games>>().unwrap();
    let mut map = mutex.lock().unwrap();
    let key: String = req.extensions.get::<router::Router>().unwrap().find("gameId").unwrap().into();

    let mut game = match map.get_mut(&key) {
        Some(game) => game,
        None => return respond_html("No such game".into()),
    };

    let turn: String = match req.get_ref::<params::Params>() {
        Ok(map) => match map.get("turn") {
            Some(turn) => match turn {
                &params::Value::String(ref s) => s.clone(),
                _ => return respond_html("Bad parsing".into()),
            },
            None => return respond_html("Bad parsing".into()),
        },
        Err(_) => return respond_html("Bad parsing".into()),
    };

    match game.game.play(&turn) {
        Ok(_) => respond_game(&game.game),
        Err(_) => respond_html(format!("Illegal move: {}", turn)),
    }
}
fn view_game(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, Path::new("html/three.html"))))
}

fn main() {
    let router = router!(get "/" => list_games,
                         get "/game/:gameId" => view_game,
                         get "/game/:gameId/player" => claim_player,
                         get "/game/:gameId/json" => game_json,
                         get "/game/:gameId/text" => serve_game,
                         post "/game/:gameId" => play_move);
    let mut chain = Chain::new(router);
    chain.link(Write::<Games>::both(HashMap::new()));
    let mut mount = Mount::new();
    mount.mount("/js", Static::new(Path::new("js/")));
    //mount.mount("/three", Static::new(Path::new("html/three.html")));
    mount.mount("/", chain);
    Iron::new(mount).http(("0.0.0.0", get_server_port())).unwrap();
}
