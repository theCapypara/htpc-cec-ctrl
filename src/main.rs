use crate::cgroup::user_slice_unlimit_cpu;
use log::info;
use std::env;

mod cec;
mod cgroup;
mod httpserver;
mod input;

#[tokio::main]
async fn main() {
    pretty_env_logger::init_timed();
    info!("start");

    let flag: Option<String> = env::args().nth(1);
    if flag.as_deref() == Some("unrestrict-cpu") {
        info!("unrestricting cpu, then exiting");
        user_slice_unlimit_cpu().ok();
        return;
    }

    let loaded_input = input::setup_input();
    info!("input setup");

    let cec_connection = cec::run_cec(loaded_input);
    info!("cec setup");

    httpserver::run_server(cec_connection).await;
}
