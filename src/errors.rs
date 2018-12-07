use chrono::ParseError;
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
