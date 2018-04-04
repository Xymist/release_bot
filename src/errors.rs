use chrono::ParseError;
use reqwest;
use std::num;
use zohohorrorshow;

error_chain!{
    foreign_links {
        NumError(num::ParseIntError);
        Zohohorrorshow(zohohorrorshow::errors::Error);
        Reqwest(reqwest::Error);
        Chrono(ParseError);
    }
}
