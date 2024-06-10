// use axum::{
//     extract::ws::{WebSocket, WebSocketUpgrade, Message},
//     http::Method,
//     response::IntoResponse,
//     routing::get,
//     Router,
// };
// use serde::{Deserialize, Serialize};
// use tower_http::cors::{CorsLayer, Any};
// use std::sync::{Arc, Mutex};


// #[derive(Serialize, Deserialize)]
// struct Position {
//     row: usize,
//     column: usize,
// }

// struct AppState {
//     position: Arc<Mutex<Position>>,
// }

// async fn websocket_handler(ws: WebSocketUpgrade, state: Arc<Mutex<Position>>) -> impl IntoResponse {
//     ws.on_upgrade(move|socket| websocket(socket, state))
// }

// async fn websocket(stream: WebSocket, state: Arc<Mutex<Position>>) {
//     use futures_util::StreamExt;

//     let (mut sender, mut receiver) = stream.split();

//     while let Some(Ok(message)) = receiver.next().await {
//         if let Message::Text(text) = message {
//             if let Ok(movement) = serde_json::from_str::<Movement>(&text) {
//                 let mut position = state.lock().unwrap();
//                 match movement.direction.as_str() {
//                     "ArrowUp" => if position.row > 1 { position.row -= 1 },
//                     "ArrowDown" => if position.row < 10 { position.row += 1 },
//                     "ArrowLeft" => if position.column > 1 { position.column -= 1 },
//                     "ArrowRight" => if position.column < 10 { position.column += 1 },
//                     _ => (),
//                 }
//                 let _ = sender.send(Message::Text(serde_json::to_string(&*position).unwrap())).await;
//             }
//         }
//     }
// }

// #[derive(Deserialize)]
// struct Movement {
//     direction: String,
// }

// #[tokio::main]
// async fn main() {
//     let initial_position = Position { row: 1, column: 1 };
//     let app_state = Arc::new(Mutex::new(initial_position));

//     let cors = CorsLayer::new()
//         .allow_origin(Any)
//         .allow_methods([Method::GET, Method::POST])
//         .allow_headers(Any);

//     let app = Router::new()
//         .route("/ws", get(websocket_handler))
//         .layer(cors)
//         .with_state(app_state);

//     axum_server::bind("127.0.0.1:8080".parse().unwrap())
//         .serve(app.into_make_service())
//         .await
//         .unwrap();
// }

use axum::{
    extract::State, 
    response::{IntoResponse, Json}, 
    routing::{get, post}, 
    http::{Method, StatusCode},
    Router};
use serde::{Serialize, Deserialize};
use tower_http::cors::{Any, CorsLayer};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;


#[derive(Serialize, Clone)]
struct Position {
    row: usize,
    column: usize,
}

#[derive(Deserialize, Clone)]
struct Movement {
    direction: String
}
// enum Movement {
//     UP,
//     DOWN,
//     LEFT,
//     RIGHT,
//     NONE,
// }

#[derive(Clone)] // For some reason this is required for the route call...
struct AppState {
    position: Arc<Mutex<Position>>,
    row_bound: usize,
    column_bound: usize,
}

#[tokio::main]
async fn main() {
    let app_state = AppState{ 
        position: Arc::new(Mutex::new(Position { row: 5, column: 5})),
        row_bound: 10,
        column_bound: 10,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    let app = Router::new()
        .route("/position", get(get_position))
        .route("/move", post(move_position))
        .layer(cors)
        .with_state(app_state);

    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_position(State(state): State<AppState>) -> Json<Position> {
    let position = state.position.lock().unwrap().clone();
    return Json(position)
}

async fn move_position(
    State(state): State<AppState>,
    Json(movement): Json<Movement>,
) -> Json<Position> {
    let mut position = state.position.lock().unwrap();
    match movement.direction.as_str() {
        "ArrowUp" => position.row -= 1, // TODO: need to check bounds here eventually, and probably tie the board size to AppState
        "ArrowDown" => position.row += 1,
        "ArrowLeft" => position.column -= 1,
        "ArrowRight" => position.column += 1,
        // Movement::UP => position.column += 1, // TODO: need to check bounds here eventually, and probably tie the board size to AppState
        // Movement::DOWN => position.column -= 1,
        // Movement::LEFT => position.row += 1,
        // Movement::RIGHT => position.row -= 1,
        _ => (),
    }
    if position.row >= state.row_bound { position.row = state.row_bound; }
    if position.row <= 1 { position.row = 1; }
    if position.column >= state.column_bound { position.column = state.column_bound; }
    if position.column <= 1 { position.column = 1; }
    Json(position.clone())
}