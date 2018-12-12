use chrono::ParseError;
use error_chain::{
    error_chain, error_chain_processing, impl_error_chain_kind, impl_error_chain_processed,
    impl_extract_backtrace,
};
use reqwest;
use std::{io, num};
use zohohorrorshow;

error_chain! {
    foreign_links {
        NumError(num::ParseIntError);
        IOError(io::Error);
        Zohohorrorshow(zohohorrorshow::errors::Error);
        Reqwest(reqwest::Error);
        Chrono(ParseError);
    }
}
