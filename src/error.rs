use postgres;
use reqwest;
use serde_yaml;

macro_rules! from {
    ($root:path, $destination_enum:ident :: $path_:ident) => {
        impl From<$root> for $destination_enum {
            fn from(e: $root) -> Self {
                $destination_enum::$path_(e)
            }
        }
    };
}

#[derive(Debug)]
pub enum Error {
    SerdeYaml(serde_yaml::Error),
    IOError(std::io::Error),
    Reqwest(reqwest::Error),
    Postgres(postgres::Error),
    DatabaseParse,
    EmptyQuery,
}

from! {serde_yaml::Error, Error::SerdeYaml}
from! {std::io::Error, Error::IOError}
from! {reqwest::Error, Error::Reqwest}
from! {postgres::Error, Error::Postgres}
