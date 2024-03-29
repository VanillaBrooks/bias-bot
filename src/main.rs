mod data;
mod discord;
mod error;
use std::sync::Arc;

fn main() {
    let anime = "./anime.yaml".to_string();
    let config = std::path::Path::new("./config.yaml");

    let info = data::config::get_data_handler(anime);

    dbg! {&info};

    let i = info.unwrap();

    dbg! {discord::bot::start_bot(&config, i)};
}
