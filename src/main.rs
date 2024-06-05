use actix_cors::Cors;
use actix_web::{web, App, HttpServer, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Debug)]
struct Position {
    row: usize,
    column: usize,
}

struct AppState {
    position: Mutex<Position>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let initial_position = Position { row: 1, column: 1 };
    let app_state = web::Data::new(AppState {
        position: Mutex::new(initial_position),
    });

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();
        
        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .route("/position", web::get().to(get_position))
            .route("/move", web::post().to(move_square))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

async fn get_position(data: web::Data<AppState>) -> HttpResponse {
    let position = data.position.lock().unwrap();
    println!("Returning position: {:?}", *position);
    HttpResponse::Ok().json(&*position)
}

async fn move_square(data: web::Data<AppState>, movement: web::Json<Movement>) -> HttpResponse {
    let mut position = data.position.lock().unwrap();
    println!("Received move request: {:?}", movement.direction);
    match movement.direction.as_str() {
        "ArrowUp" => if position.row > 1 { position.row -= 1 },
        "ArrowDown" => if position.row < 10 { position.row += 1 },
        "ArrowLeft" => if position.column > 1 { position.column -= 1 },
        "ArrowRight" => if position.column < 10 { position.column += 1 },
        _ => (),
    }
    println!("New position: {:?}", *position);
    HttpResponse::Ok().json(&*position)
}

#[derive(Deserialize)]
struct Movement {
    direction: String,
}