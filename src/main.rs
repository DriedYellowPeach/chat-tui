pub mod action;
pub mod app;
pub mod models;
pub mod tio;
pub mod ui;

#[tokio::main]
async fn main() {
    let mut app = app::App::new().unwrap();
    app.run().await.unwrap();
}
