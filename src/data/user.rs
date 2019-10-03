use reqwest;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_yaml;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Data {
    pub shows: Vec<Anime>,
    pub save_location: String,
}

impl Data {
    pub fn path(&self) -> &Path {
        Path::new(&self.save_location)
    }
    pub fn shows(&self) -> &Vec<Anime> {
        &self.shows
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Anime {
    pub name: String,
    pub characters: Vec<Character>,
}
impl Anime {
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn characters(&self) -> &Vec<Character> {
        &self.characters
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Character {
    pub name: String,
    pub links: Vec<String>,
}
impl Character {
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn links(&self) -> &Vec<String> {
        &self.links
    }
}
