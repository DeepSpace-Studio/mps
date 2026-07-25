use mps_web::app;

#[tokio::main]
async fn main() {
    topcoat::start(app()).await.unwrap();
}