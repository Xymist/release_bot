use reqwest;
use chrono::ParseError;
error_chain!{
    foreign_links {
        Reqwest(reqwest::Error);
        Chrono(ParseError);
    }
}
