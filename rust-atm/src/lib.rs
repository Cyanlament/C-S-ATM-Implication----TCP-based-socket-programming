use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub const RESP_AUTH_REQUIRE: &str = "500 AUTH REQUIRE";
pub const RESP_OK: &str = "525 OK!";
pub const RESP_ERROR: &str = "401 ERROR!";
pub const RESP_BYE: &str = "BYE";

pub type UsersDb = HashMap<String, String>;
pub type BalancesDb = HashMap<String, f64>;

#[derive(Debug, Clone, PartialEq)]
pub enum Request {
    Helo(String),
    Pass(String),
    Bala,
    Wdra(f64),
    Quit,
}

pub fn parse_request(line: &str) -> Option<Request> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut parts = trimmed.split_whitespace();
    let cmd = parts.next()?.to_uppercase();

    match cmd.as_str() {
        "HELO" => {
            let user_id = parts.next()?;
            if parts.next().is_some() {
                None
            } else {
                Some(Request::Helo(user_id.to_string()))
            }
        }
        "PASS" => {
            let password = parts.next()?;
            if parts.next().is_some() {
                None
            } else {
                Some(Request::Pass(password.to_string()))
            }
        }
        "BALA" => {
            if parts.next().is_some() {
                None
            } else {
                Some(Request::Bala)
            }
        }
        "WDRA" => {
            let amount_str = parts.next()?;
            if parts.next().is_some() {
                return None;
            }
            let amount = amount_str.parse::<f64>().ok()?;
            Some(Request::Wdra(amount))
        }
        "QUIT" => {
            if parts.next().is_some() {
                None
            } else {
                Some(Request::Quit)
            }
        }
        _ => None,
    }
}

pub fn format_amount_response(amount: f64) -> String {
    format!("AMNT:{amount:.2}")
}

pub fn load_users(path: &Path) -> io::Result<UsersDb> {
    let content = fs::read_to_string(path)?;
    let mut db = UsersDb::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() != 2 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid users.txt line: {trimmed}"),
            ));
        }

        db.insert(parts[0].to_string(), parts[1].to_string());
    }

    Ok(db)
}

pub fn load_balances(path: &Path) -> io::Result<BalancesDb> {
    let content = fs::read_to_string(path)?;
    let mut db = BalancesDb::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() != 2 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid balances.txt line: {trimmed}"),
            ));
        }

        let amount = parts[1].parse::<f64>().map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid balance `{}`: {error}", parts[1]),
            )
        })?;

        db.insert(parts[0].to_string(), amount);
    }

    Ok(db)
}

pub fn save_balances(path: &Path, balances: &BalancesDb) -> io::Result<()> {
    let mut rows: Vec<_> = balances.iter().collect();
    rows.sort_by(|a, b| a.0.cmp(b.0));

    let mut content = String::new();
    for (user_id, amount) in rows {
        content.push_str(&format!("{user_id} {amount:.2}\n"));
    }

    fs::write(path, content)
}

pub fn now_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

pub fn append_log(log_file: &Path, msg: &str) -> io::Result<()> {
    if let Some(parent) = log_file.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)?;

    writeln!(file, "[{}] {}", now_epoch_secs(), msg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_helo_ok() {
        assert_eq!(
            parse_request("HELO 100001"),
            Some(Request::Helo("100001".to_string()))
        );
    }

    #[test]
    fn parse_wdra_ok() {
        assert_eq!(parse_request("WDRA 30"), Some(Request::Wdra(30.0)));
    }

    #[test]
    fn parse_quit_ok() {
        assert_eq!(parse_request("QUIT"), Some(Request::Quit));
    }

    #[test]
    fn parse_invalid_extra_args() {
        assert_eq!(parse_request("BALA now"), None);
    }
}
