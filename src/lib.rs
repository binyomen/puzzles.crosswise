#![feature(proc_macro_hygiene, decl_macro)]

extern crate reqwest;
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
        .map_err(|_e| String::from("invalid month value"))
        .and_then(|m| {
            if m >= 1 && m <= 12 {
                Ok(m)
            } else {
                Err(String::from("month value out of range"))
            }
        })?;

    let day = tokens[3]
        .parse::<u8>()
        .map_err(|_e| String::from("invalid day value"))
        .and_then(|d| {
            if d == 0 {
                return Err(String::from("0 is not a valid day value"));
            }

            let check_day_validity = |max_day| {
                let err_prefix = String::from("day must be less than or equal to ");
                if d <= max_day {
                    Ok(d)
                } else {
                    Err(format!("{}{}", err_prefix, max_day))
                }
            };
            match month {
                1 | 3 | 5 | 7 | 8 | 10 | 12 => check_day_validity(31),
                4 | 6 | 9 | 11 => check_day_validity(30),
                // TODO: handle leap years
                2 => check_day_validity(29),
                0 | 12...255 => unreachable!(),
            }
        })?;

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

fn retrieve_url(url: String) -> reqwest::Result<String> {
    Ok(reqwest::get(&url)?.error_for_status()?.text()?)
}

#[get("/<id>")]
fn get_puzzle(id: PuzzleId) -> Option<String> {
    let url = id_to_url(id);
    retrieve_url(url).ok()
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
    fn invalid_id_length_fails() {
        assert_eq!(
            parse_id(String::from("lat-2019-1-1-1")).unwrap_err(),
            "invalid ID length"
        );
    }

    #[test]
    fn invalid_source_id_fails() {
        assert_eq!(
            parse_id(String::from("foo-2019-1-1")).unwrap_err(),
            "invalid source ID"
        );
    }

    #[test]
    fn invalid_year_value_fails() {
        assert_eq!(
            parse_id(String::from("lat-a-b-c")).unwrap_err(),
            "invalid year value"
        );
    }

    #[test]
    fn invalid_month_value_fails() {
        assert_eq!(
            parse_id(String::from("lat-1-b-c")).unwrap_err(),
            "invalid month value"
        );
        assert_eq!(
            parse_id(String::from("lat-1-0-1")).unwrap_err(),
            "month value out of range"
        );
        assert_eq!(
            parse_id(String::from("lat-1-13-1")).unwrap_err(),
            "month value out of range"
        );
    }

    #[test]
    fn invalid_day_value_fails() {
        assert_eq!(
            parse_id(String::from("lat-1-2-c")).unwrap_err(),
            "invalid day value"
        );
        assert_eq!(
            parse_id(String::from("lat-1-1-0")).unwrap_err(),
            "0 is not a valid day value"
        );
        assert_eq!(
            parse_id(String::from("lat-1-1-32")).unwrap_err(),
            "day must be less than or equal to 31"
        );
        assert_eq!(
            parse_id(String::from("lat-1-4-31")).unwrap_err(),
            "day must be less than or equal to 30"
        );
        assert_eq!(
            parse_id(String::from("lat-1-2-30")).unwrap_err(),
            "day must be less than or equal to 29"
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
