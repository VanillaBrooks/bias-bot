use serenity;

mod data;
mod error;

fn main() {
    let info = data::config::get_files("config.yaml");

    dbg! {info};
}
