#![feature(proc_macro_hygiene, decl_macro)]

mod redis_helper;

use types::*;

use reqwest::blocking;
use rocket::get;
use rocket::http::ContentType;
use rocket::response::content::Content;
use rocket::routes;

mod types {
    use std::convert::TryInto;
    use std::fmt;

    use rocket::http::RawStr;
    use rocket::request::FromParam;
    use serde::{Deserialize, Serialize};

    // from https://stackoverflow.com/a/11595914
    fn is_leap_year(year: u16) -> bool {
        year.trailing_zeros() >= 2 && ((year % 25) != 0 || year.trailing_zeros() >= 4)
    }

    fn parse_id(id: String) -> Result<PuzzleId, String> {
        let tokens_vec: Vec<&str> = id.split('-').collect();
        let tokens: &[&str; 4] = tokens_vec
            .as_slice()
            .try_into()
            .map_err(|_e| String::from("invalid ID length"))?;

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
                if (1..=12).contains(&m) {
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
                    2 => check_day_validity(if is_leap_year(year) { 29 } else { 28 }),
                    0 | 13..=255 => unreachable!(),
                }
            })?;

        Ok(PuzzleId {
            source_id,
            date: PuzzleDate { year, month, day },
        })
    }

    #[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
    pub enum PuzzlesContentType {
        Xml,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct PuzzlesContent {
        pub content: String,
        pub content_type: PuzzlesContentType,
    }

    #[derive(Debug, PartialEq, Eq, Serialize)]
    pub enum SourceId {
        LaTimes,
    }

    impl fmt::Display for SourceId {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    SourceId::LaTimes => "lat",
                }
            )
        }
    }

    #[derive(Debug, PartialEq, Eq, Serialize)]
    pub struct PuzzleDate {
        pub year: u16,
        pub month: u8,
        pub day: u8,
    }

    #[derive(Debug, PartialEq, Eq, Serialize)]
    pub struct PuzzleId {
        pub source_id: SourceId,
        pub date: PuzzleDate,
    }

    impl fmt::Display for PuzzleId {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(
                f,
                "{}-{}-{}-{}",
                self.source_id, self.date.year, self.date.month, self.date.day
            )
        }
    }

    impl<'a> FromParam<'a> for PuzzleId {
        type Error = &'a RawStr;
        fn from_param(param: &'a RawStr) -> Result<Self, Self::Error> {
            let param_string = String::from_param(param)?;
            parse_id(param_string).map_err(|_e| param)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn correctly_determines_leap_year() {
            assert_eq!(is_leap_year(2012), true);
            assert_eq!(is_leap_year(1804), true);
            assert_eq!(is_leap_year(2000), true);

            assert_eq!(is_leap_year(2011), false);
            assert_eq!(is_leap_year(1806), false);
            assert_eq!(is_leap_year(1900), false);
        }

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
                parse_id(String::from("lat-2012-2-30")).unwrap_err(),
                "day must be less than or equal to 29"
            );
            assert_eq!(
                parse_id(String::from("lat-2011-2-29")).unwrap_err(),
                "day must be less than or equal to 28"
            );
        }
    }
}

fn id_to_url(id: &PuzzleId) -> (String, PuzzlesContentType) {
    match id.source_id {
        SourceId::LaTimes => (
            String::from("http://cdn.games.arkadiumhosted.com/latimes/assets/DailyCrossword/")
                + "la"
                + &format!("{:02}", id.date.year % 100)
                + &format!("{:02}", id.date.month)
                + &format!("{:02}", id.date.day)
                + ".xml",
            PuzzlesContentType::Xml,
        ),
    }
}

fn retrieve_url(url: String) -> reqwest::Result<String> {
    blocking::get(&url)?.error_for_status()?.text()
}

#[get("/<id>")]
fn get_puzzle(id: PuzzleId) -> Option<Content<String>> {
    let content = match redis_helper::fetch_puzzle_from_cache(&id) {
        Some(content) => content,
        None => {
            let (url, content_type) = id_to_url(&id);
            let response_string = retrieve_url(url).ok()?;
            let content = PuzzlesContent {
                content: response_string,
                content_type,
            };
            redis_helper::put_puzzle_into_cache(&id, &content);
            content
        }
    };

    Some(Content(
        match content.content_type {
            PuzzlesContentType::Xml => ContentType::XML,
        },
        content.content,
    ))
}

pub fn start_server() {
    rocket::ignite().mount("/", routes![get_puzzle]).launch();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gets_url() {
        assert_eq!(
            id_to_url(&PuzzleId {
                source_id: SourceId::LaTimes,
                date: PuzzleDate {
                    year: 2019,
                    month: 1,
                    day: 2
                }
            }),
            (
                String::from("http://cdn.games.arkadiumhosted.com/latimes/assets/DailyCrossword/la190102.xml"),
                PuzzlesContentType::Xml
            )
        );
        assert_eq!(
            id_to_url(&PuzzleId {
                source_id: SourceId::LaTimes,
                date: PuzzleDate {
                    year: 13,
                    month: 12,
                    day: 28
                }
            }),
            (
                String::from("http://cdn.games.arkadiumhosted.com/latimes/assets/DailyCrossword/la131228.xml"),
                PuzzlesContentType::Xml
            )
        );
        assert_eq!(
            id_to_url(&PuzzleId {
                source_id: SourceId::LaTimes,
                date: PuzzleDate {
                    year: 1,
                    month: 12,
                    day: 28
                }
            }),
            (
                String::from("http://cdn.games.arkadiumhosted.com/latimes/assets/DailyCrossword/la011228.xml"),
                PuzzlesContentType::Xml
            )
        );
    }
}
