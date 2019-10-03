//! Reads file that need to be downloaded from the internet and places them in the file system
//!

use reqwest;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_yaml;

use super::super::error::Error;
use std::fs;
use std::io::prelude::*;

use super::{cpu, user};

pub fn get_files(path: &str) -> Result<cpu::Data, Error> {
    let reader = fs::File::open(path)?;

    let ser_data: user::Data = serde_yaml::from_reader(reader)?;

    // let downloader = Downloader::new(ser_data.path());
    let mut previous_data = get_older_data(ser_data)?;
    // let mut previous_hm = previous_data.to_hashmap();

    // Ok(ser_data)

    unimplemented!()
}

fn get_older_data<'a>(data: user::Data) -> Result<cpu::RefData<'a>, Error> {
    let save_location = data.path();
    // create the directory if it does not exist
    if !save_location.is_dir() {
        fs::create_dir_all(save_location)?;
    }

    let file_path = save_location.join("previously_saved.yaml");

    let previous = match file_path.is_file() {
        true => {
            let file = fs::File::open(&file_path)?;
            let ser: Result<cpu::Data, _> = serde_yaml::from_reader(file);
            match ser {
                Ok(ser_data) => ser_data.to_ref(data)?,
                Err(_) => {
                    println! {"error when reading from the generated config file"};
                    cpu::RefData::new()
                }
            }
        }
        false => {
            fs::File::create(&file_path)?;
            cpu::RefData::new()
        }
    };

    Ok(previous)
}
