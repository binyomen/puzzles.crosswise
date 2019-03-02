extern crate bincode;
extern crate redis;

use crate::types::{PuzzleId, PuzzlesContent};

use redis::{Commands, FromRedisValue, RedisError, RedisResult, ToRedisArgs, Value};

impl ToRedisArgs for &PuzzleId {
    fn write_redis_args(&self, out: &mut Vec<Vec<u8>>) {
        out.push(bincode::serialize(&self).unwrap());
    }
}

impl ToRedisArgs for &PuzzlesContent {
    fn write_redis_args(&self, out: &mut Vec<Vec<u8>>) {
        out.push(bincode::serialize(&self).unwrap());
    }
}

impl FromRedisValue for PuzzlesContent {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        match *v {
            Value::Data(ref bytes) => Ok(bincode::deserialize(&bytes).map_err(|_e| {
                RedisError::from((redis::ErrorKind::TypeError, "failed to deserialize"))
            })?),
            _ => Err(RedisError::from((
                redis::ErrorKind::TypeError,
                "response type not string compatible",
            ))),
        }
    }
}

fn truncate_string(s: &str, length: usize) -> &str {
    if s.len() < length {
        s
    } else {
        &s[..length]
    }
}

pub fn fetch_puzzle_from_cache(id: &PuzzleId) -> Option<PuzzlesContent> {
    let client = redis::Client::open("redis://redis").ok()?;
    let connection = client.get_connection().ok()?;

    let content: PuzzlesContent = connection.get(id).ok()?;

    println!(
        "fetched from redis: {}: {}...",
        id,
        truncate_string(&content.content, 10)
    );
    Some(content)
}

pub fn put_puzzle_into_cache(id: &PuzzleId, content: &PuzzlesContent) {
    let client = match redis::Client::open("redis://redis") {
        Ok(v) => v,
        Err(_) => return,
    };

    let connection = match client.get_connection() {
        Ok(v) => v,
        Err(_) => return,
    };

    const TWO_DAYS: usize = 60 * 60 * 24 * 2;
    match connection.set_ex(id, content, TWO_DAYS) {
        Ok(()) => println!(
            "put into redis: {}: {}...",
            id,
            truncate_string(&content.content, 10)
        ),
        Err(_) => return,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncates_string() {
        assert_eq!(truncate_string("a", 1), "a");
        assert_eq!(truncate_string("abcde", 1), "a");
        assert_eq!(truncate_string("abcde", 4), "abcd");
        assert_eq!(truncate_string("abcde", 10), "abcde");
    }
}