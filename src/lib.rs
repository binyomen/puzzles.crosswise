#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket::http::RawStr;
use rocket::request::FromParam;

#[derive(Debug, PartialEq)]
enum SourceId {
    LaTimes,
}

#[derive(Debug, PartialEq)]
struct PuzzleDate {
    year: u16,
    month: u8,
    day: u8,
}

#[derive(Debug, PartialEq)]
struct PuzzleId {
    source_id: SourceId,
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

    let source_id = match tokens[0] {
        "lat" => SourceId::LaTimes,
        _ => return Err(String::from("invalid source ID")),
    };

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
        source_id: source_id,
        date: PuzzleDate {
            year: year,
            month: month,
            day: day,
        },
    })
}

fn id_to_url(id: PuzzleId) -> String {
    match id.source_id {
        SourceId::LaTimes => {
            String::from("http://cdn.games.arkadiumhosted.com/latimes/assets/DailyCrossword/")
                + "la"
                + &format!("{:02}", id.date.year % 100)
                + &format!("{:02}", id.date.month)
                + &format!("{:02}", id.date.day)
                + ".xml"
        }
    }
}

#[get("/<id>")]
fn get_puzzle(id: PuzzleId) -> String {
    let url = id_to_url(id);
    format!("{}", url)
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
            parse_id(String::from("lat-2019-1-2")).unwrap(),
            PuzzleId {
                source_id: SourceId::LaTimes,
                date: PuzzleDate {
                    year: 2019,
                    month: 1,
                    day: 2,
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
            parse_id(String::from("foo-2019-1-1")).unwrap_err(),
            "invalid source ID"
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

    #[test]
    fn gets_url() {
        assert_eq!(
            id_to_url(PuzzleId {
                source_id: SourceId::LaTimes,
                date: PuzzleDate {
                    year: 2019,
                    month: 1,
                    day: 2
                }
            }),
            "http://cdn.games.arkadiumhosted.com/latimes/assets/DailyCrossword/la190102.xml"
        );

        assert_eq!(
            id_to_url(PuzzleId {
                source_id: SourceId::LaTimes,
                date: PuzzleDate {
                    year: 13,
                    month: 12,
                    day: 28
                }
            }),
            "http://cdn.games.arkadiumhosted.com/latimes/assets/DailyCrossword/la131228.xml"
        );

        assert_eq!(
            id_to_url(PuzzleId {
                source_id: SourceId::LaTimes,
                date: PuzzleDate {
                    year: 1,
                    month: 12,
                    day: 28
                }
            }),
            "http://cdn.games.arkadiumhosted.com/latimes/assets/DailyCrossword/la011228.xml"
        );
    }
}
