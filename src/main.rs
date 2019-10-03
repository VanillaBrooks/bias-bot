use serenity;

mod data;
mod error;

fn main() {
    let p = std::path::Path::new("./config.yaml");
    let info = data::config::get_data_handler(&p);

    dbg! {info};
}
