use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub const RESP_AUTH_REQUIRED: &str = "500 AUTH REQUIRED!";
pub const RESP_OK: &str = "525 OK!";
pub const RESP_ERROR: &str = "401 ERROR!";
pub const RESP_BYE: &str = "BYE";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub password: String,
    pub balance: f64,
}

pub type AccountsDb = HashMap<String, Account>;

#[derive(Debug, Clone, PartialEq)]
pub enum Request {
    Helo(String),
    Pass(String),
    Bala,
    Wdra(f64),
    Bye,
}

pub fn parse_request(line: &str) -> Option<Request> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut parts = trimmed.split_whitespace();
    let cmd = parts.next()?.to_uppercase();

    match cmd.as_str() {
        "HELO" => parts.next().map(|s| Request::Helo(s.to_string())),
        "PASS" => parts.next().map(|s| Request::Pass(s.to_string())),
        "BALA" => Some(Request::Bala),
        "WDRA" => {
            let amount_str = parts.next()?;
            let amount = amount_str.parse::<f64>().ok()?;
            Some(Request::Wdra(amount))
        }
        "BYE" => Some(Request::Bye),
        _ => None,
    }
}

pub fn format_amount_response(amount: f64) -> String {
    format!("AMNT:{amount:.2}")
}

pub fn load_accounts(path: &Path) -> io::Result<AccountsDb> {
    let content = fs::read_to_string(path)?;
    let db: AccountsDb = serde_json::from_str(&content)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    Ok(db)
}

pub fn save_accounts(path: &Path, db: &AccountsDb) -> io::Result<()> {
    let content = serde_json::to_string_pretty(db)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    fs::write(path, content)
}

pub fn now_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
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
        assert_eq!(parse_request("HELO 10001"), Some(Request::Helo("10001".to_string())));
    }

    #[test]
    fn parse_wdra_ok() {
        assert_eq!(parse_request("WDRA 30"), Some(Request::Wdra(30.0)));
    }

    #[test]
    fn parse_invalid() {
        assert_eq!(parse_request("XYZ"), None);
    }
}
