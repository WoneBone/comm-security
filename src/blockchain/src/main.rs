// Remove the following 3 lines to enable compiler checkings
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use axum::{
    extract::Extension,
    response::{sse::Event, Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use futures::stream::StreamExt;
use rand::{seq::IteratorRandom, SeedableRng};
use risc0_zkvm::Digest;
use std::{
    collections::HashMap,
    error::Error,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

use fleetcore::{BaseJournal, Command, FireJournal, CommunicationData, ReportJournal};
use methods::{FIRE_ID, JOIN_ID, REPORT_ID, WAVE_ID, WIN_ID};

struct Player {
    name: String,
    current_state: Digest,
}
struct Game {
    pmap: HashMap<String, Player>,
    next_player: Option<String>,
    next_report: Option<String>,
}

#[derive(Clone)]
struct SharedData {
    tx: broadcast::Sender<String>,
    gmap: Arc<Mutex<HashMap<String, Game>>>,
    rng: Arc<Mutex<rand::rngs::StdRng>>,
}

#[tokio::main]
async fn main() {
    // Create a broadcast channel for log messages
    let (tx, _rx) = broadcast::channel::<String>(100);
    let shared = SharedData {
        tx: tx,
        gmap: Arc::new(Mutex::new(HashMap::new())),
        rng: Arc::new(Mutex::new(rand::rngs::StdRng::from_entropy())),
    };

    // Build our application with a route

    let app = Router::new()
        .route("/", get(index))
        .route("/logs", get(logs))
        .route("/chain", post(smart_contract))
        .layer(Extension(shared));

    // Run our app with hyper
    //let addr = SocketAddr::from(([127, 0, 0, 1], 3001));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    println!("Listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Handler to serve the HTML page
async fn index() -> Html<&'static str> {
    Html(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Blockchain Emulator</title>
        </head>
        <body>
            <h1>Registered Transactions</h1>          
            <ul id="logs"></ul>
            <script>
                const eventSource = new EventSource('/logs');
                eventSource.onmessage = function(event) {
                    const logs = document.getElementById('logs');
                    const log = document.createElement('li');
                    log.textContent = event.data;
                    logs.appendChild(log);
                };
            </script>
        </body>
        </html>
        "#,
    )
}

// Handler to manage SSE connections
#[axum::debug_handler]
async fn logs(Extension(shared): Extension<SharedData>) -> impl IntoResponse {
    let rx = BroadcastStream::new(shared.tx.subscribe());
    let stream = rx.filter_map(|result| async move {
        match result {
            Ok(msg) => Some(Ok(Event::default().data(msg))),
            Err(_) => Some(Err(Box::<dyn Error + Send + Sync>::from("Error"))),
        }
    });

    axum::response::sse::Sse::new(stream)
}

fn xy_pos(pos: u8) -> String {
    let x = pos % 10;
    let y = pos / 10;
    format!("{}{}", (x + 65) as char, y)
}

async fn smart_contract(
    Extension(shared): Extension<SharedData>,
    Json(input_data): Json<CommunicationData>,
) -> String {
    match input_data.cmd {
        Command::Join => handle_join(&shared, &input_data),
        Command::Fire => handle_fire(&shared, &input_data),
        Command::Report => handle_report(&shared, &input_data),
        Command::Wave => handle_wave(&shared, &input_data),
        Command::Win => handle_win(&shared, &input_data),
    }
}

fn handle_join(shared: &SharedData, input_data: &CommunicationData) -> String {
    if input_data.receipt.verify(JOIN_ID).is_err() {
        shared.tx.send("Attempting to join game with invalid receipt".to_string()).unwrap();
        return "Could not verify receipt".to_string();
    }
    let data: BaseJournal = input_data.receipt.journal.decode().unwrap();
    let mut gmap = shared.gmap.lock().unwrap();
    let game = gmap.entry(data.gameid.clone()).or_insert(Game {
        pmap: HashMap::new(),
        next_player: Some(data.fleet.clone()),
        next_report: None,
    });
    let player_inserted = game.pmap.entry(data.fleet.clone()).or_insert_with(|| Player {
        name: data.fleet.clone(),
        current_state: data.board.clone(),
    }).name == data.fleet;
    let mesg = if player_inserted {
        format!("Joined game {}", data.gameid)
    } else {
        format!("Player already in game {}", data.gameid)
    };
    shared.tx.send(mesg).unwrap();
    "OK".to_string()
}

fn handle_fire(shared: &SharedData, input_data: &CommunicationData) -> String {
    if input_data.receipt.verify(FIRE_ID).is_err() {
        shared.tx.send("Attempting to fire with invalid receipt".to_string()).unwrap();
        return "Could not verify receipt".to_string();
    }

    let data: FireJournal = input_data.receipt.journal.decode().unwrap();

    let mut gmap = shared.gmap.lock().unwrap();
    if let Some(game) = gmap.get_mut(&data.gameid) { //get the game with game id

        if game.pmap.contains_key(&data.fleet) { //check if the fleet exists
            if game.next_player == Some(data.fleet.clone()) { //check if it is this players turn
                if game.next_report.is_none() { //check if the previous report has been addressed
                    if game.pmap.contains_key(&data.target) { //check if the fleet exists
                        game.next_player = Some(data.target.clone());
                        game.next_report = Some(data.pos.to_string());
                        let x = (data.pos % 10 + b'A') as char;
                        let y = (data.pos / 10);
                        let msg = format!("Player {} fired at player {} at pos {}.{}", data.fleet, data.target, 
                            x, y);
                        shared.tx.send(msg).unwrap();
                    }

                    else {
                        let msg = format!("Player {} not in game", data.target);
                        shared.tx.send(msg).unwrap();
                    }
                }
                else {
                    let msg = format!("Must address report first");
                    shared.tx.send(msg).unwrap();
                }
            }
            else {
                let msg = format!("Not your turn dummy");
                shared.tx.send(msg).unwrap();
            }
        }
        else {
            let msg = format!("Player {} not in game", data.fleet);
            shared.tx.send(msg).unwrap();
        }
    }
    else {
        let msg = format!("Game {} does not exist", data.gameid);
        shared.tx.send(msg).unwrap();
    }

    "OK".to_string()
}

fn handle_report(shared: &SharedData, input_data: &CommunicationData) -> String {
    if input_data.receipt.verify(REPORT_ID).is_err() {
        shared.tx.send("Attempting to report with invalid receipt".to_string()).unwrap();
        return "Could not verify receipt".to_string();
    }

    let data: ReportJournal = input_data.receipt.journal.decode().unwrap();

    let mut gmap = shared.gmap.lock().unwrap();
    if let Some(game) = gmap.get_mut(&data.gameid) { //get the game with game id
        if let Some(player) = game.pmap.get_mut(&data.fleet) { //check if the fleet exists
            if game.next_report.is_none() { // check if game has report
                let msg = format!("No report to handle in this game");
                shared.tx.send(msg).unwrap();
            }
            else {
                if game.next_player == Some(data.fleet.clone()) { //check if turn
                    if player.current_state == data.board { //Check if report is for the correct board
                        if game.next_report == Some(data.pos.to_string()) { //check if report is for the correct position
                                                                            
                            let x = (data.pos % 10 + b'A') as char;
                            let y = (data.pos / 10);
                            player.current_state = data.next_board.clone();
                            game.next_report = None;

                            let msg = format!("Player {} reported {} at pos {}. His updated board is {}.", data.fleet, data.report, data.pos, data.next_board);
                            shared.tx.send(msg).unwrap();
                        }
                        else {
                            let msg = format!("Report of wrong position. Shot was at pos {}", game.next_report.as_ref().unwrap_or(&"unknown".to_string()));
                            shared.tx.send(msg).unwrap();
                        }
                    }
                    else {
                        let msg = format!("Report of wrong board. Current board hash is: {}", player.current_state);
                        shared.tx.send(msg).unwrap();
                    }
                }
                else {
                    let msg = format!("Not your turn dummy");
                    shared.tx.send(msg).unwrap();
                }
            }
        }
        else {
            let msg = format!("Player {} not in game", data.fleet);
            shared.tx.send(msg).unwrap();
        }
    }
    else {
        let msg = format!("Game {} does not exist", data.gameid);
        shared.tx.send(msg).unwrap();
    }
    "OK".to_string()
}

fn handle_wave(shared: &SharedData, input_data: &CommunicationData) -> String {
    // TO DO:
    if input_data.receipt.verify(WAVE_ID).is_err() {
        shared.tx.send("Attempting to wave with invalid receipt".to_string()).unwrap();
        return "Could not verify receipt".to_string();
    }
    
    let data: BaseJournal = input_data.receipt.journal.decode().unwrap();   
    let mut gmap = shared.gmap.lock().unwrap();
    if let Some(game) = gmap.get_mut(&data.gameid) { //get the game with game id
        if game.pmap.contains_key(&data.fleet) { //check if the fleet exists
            if game.next_report.is_none() { // check if game has report
                if game.next_player == Some(data.fleet.clone()) { //check if turn
                    let msg = format!("Player {} waves their turn", data.fleet);
                    shared.tx.send(msg).unwrap();
                    if let Some((player, _))= game.pmap.iter().next(){
                        game.next_player = Some(player.clone());
                    }
                }
                else {
                    let msg = format!("Not your turn dummy");
                    shared.tx.send(msg).unwrap();
                }
            }
            else {
                let msg = format!("Must address report first");
                shared.tx.send(msg).unwrap();
            }
        }
        else {
            let msg = format!("Player {} does not exist in game {}", data.fleet, data.gameid);
            shared.tx.send(msg).unwrap();
        }
    }
    else {
        let msg = format!("Game {} does not exist", data.gameid);
        shared.tx.send(msg).unwrap();
    }

    "OK".to_string()
}

fn handle_win(shared: &SharedData, input_data: &CommunicationData) -> String {
    // TO DO:
    "OK".to_string()
}
