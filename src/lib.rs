#![feature(proc_macro_hygiene, decl_macro)]

use rocket::http::RawStr;
use rocket::request::FromParam;

#[macro_use]
extern crate rocket;

#[derive(Debug, PartialEq)]
struct PuzzleDate {
    year: u16,
    month: u8,
    day: u8,
}

#[derive(Debug, PartialEq)]
struct PuzzleId {
    source_id: String,
    date: PuzzleDate,
}

impl<'a> FromParam<'a> for PuzzleId {
    type Error = &'a RawStr;
    fn from_param(param: &'a RawStr) -> Result<Self, Self::Error> {
        let param_string = String::from_param(param)?;
        parse_id(param_string).map_err(|_e| param)
    }
}

fn parse_id(id: String) -> Result<PuzzleId, String> {
    let tokens: Vec<&str> = id.split("-").collect();
    if tokens.len() != 4 {
        return Err(String::from("invalid ID length"));
    }

    let year = tokens[1]
        .parse::<u16>()
        .map_err(|_e| String::from("invalid year value"))?;
    let month = tokens[2]
        .parse::<u8>()
        .map_err(|_e| String::from("invalid month value"))?;
    let day = tokens[3]
        .parse::<u8>()
        .map_err(|_e| String::from("invalid day value"))?;

    Ok(PuzzleId {
        source_id: String::from(tokens[0]),
        date: PuzzleDate {
            year: year,
            month: month,
            day: day,
        },
    })
}

#[get("/<id>")]
fn get_puzzle(id: PuzzleId) -> String {
    format!("{:?}", id)
}

pub fn start_server() {
    rocket::ignite().mount("/", routes![get_puzzle]).launch();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_id() {
        assert_eq!(
            parse_id(String::from("lat-2019-1-1")).unwrap(),
            PuzzleId {
                source_id: String::from("lat"),
                date: PuzzleDate {
                    year: 2019,
                    month: 1,
                    day: 1,
                }
            }
        );
    }

    #[test]
    fn invalid_id_fails() {
        assert_eq!(
            parse_id(String::from("lat-2019-1-1-1")).unwrap_err(),
            "invalid ID length"
        );

        assert_eq!(
            parse_id(String::from("lat-a-b-c")).unwrap_err(),
            "invalid year value"
        );

        assert_eq!(
            parse_id(String::from("lat-1-b-c")).unwrap_err(),
            "invalid month value"
        );

        assert_eq!(
            parse_id(String::from("lat-1-2-c")).unwrap_err(),
            "invalid day value"
        );
    }
}
