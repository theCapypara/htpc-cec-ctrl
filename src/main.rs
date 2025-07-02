use log::info;

mod cec;
mod httpserver;
mod input;

#[tokio::main]
async fn main() {
    pretty_env_logger::init_timed();
    info!("start");

    let loaded_input = input::setup_input();
    info!("input setup");

    let cec_connection = cec::run_cec(loaded_input);
    info!("cec setup");

    httpserver::run_server(cec_connection).await;
}
