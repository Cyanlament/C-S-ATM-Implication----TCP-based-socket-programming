use rust_atm::{
    append_log, format_amount_response, load_balances, load_users, parse_request, save_balances,
    BalancesDb, Request, UsersDb, RESP_AUTH_REQUIRE, RESP_BYE, RESP_ERROR, RESP_OK,
};
use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SessionState {
    Init,
    AuthRequired,
    LoggedIn,
}

#[derive(Debug)]
struct Session {
    state: SessionState,
    current_user: Option<String>,
}

impl Session {
    fn new() -> Self {
        Self {
            state: SessionState::Init,
            current_user: None,
        }
    }
}

fn write_line(stream: &mut TcpStream, line: &str) -> io::Result<()> {
    stream.write_all(line.as_bytes())?;
    stream.write_all(b"\n")?;
    stream.flush()
}

fn parse_port_arg(arg: Option<String>) -> u16 {
    let Some(raw) = arg else {
        return 2525;
    };

    match raw.trim().parse::<u16>() {
        Ok(port) if port > 0 => port,
        _ => {
            eprintln!("invalid port `{raw}`, fallback to 2525");
            2525
        }
    }
}

fn resolve_paths() -> io::Result<(PathBuf, PathBuf, PathBuf)> {
    let mut user_candidates = vec![PathBuf::from("users.txt")];
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            user_candidates.push(exe_dir.join("users.txt"));
        }
    }

    for users_path in user_candidates {
        let balance_candidate = users_path.with_file_name("balances.txt");

        if users_path.exists() && balance_candidate.exists() {
            let app_root = users_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf();
            let logs_dir = app_root.join("logs");
            return Ok((users_path, balance_candidate, logs_dir));
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "cannot find users.txt and balances.txt from current directory or executable location",
    ))
}

fn handle_client(
    mut stream: TcpStream,
    users: Arc<UsersDb>,
    balances: Arc<Mutex<BalancesDb>>,
    balances_path: PathBuf,
    logs_dir: PathBuf,
) -> io::Result<()> {
    let peer = stream
        .peer_addr()
        .map(|addr| addr.to_string())
        .unwrap_or_else(|_| "unknown-peer".to_string());

    append_log(&logs_dir.join("server.log"), &format!("client connected: {peer}"))?;

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut session = Session::new();

    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            append_log(
                &logs_dir.join("server.log"),
                &format!("client disconnected: {peer}"),
            )?;
            break;
        }

        let raw = line.trim();
        if raw.is_empty() {
            continue;
        }

        let Some(request) = parse_request(raw) else {
            append_log(
                &logs_dir.join("exception.log"),
                &format!("{peer} invalid request: {raw}"),
            )?;
            write_line(&mut stream, RESP_ERROR)?;
            continue;
        };

        match request {
            Request::Helo(user_id) => {
                if users.contains_key(&user_id) {
                    session.state = SessionState::AuthRequired;
                    session.current_user = Some(user_id);
                    write_line(&mut stream, RESP_AUTH_REQUIRE)?;
                } else {
                    session.state = SessionState::Init;
                    session.current_user = None;
                    append_log(
                        &logs_dir.join("exception.log"),
                        &format!("{peer} unknown user id"),
                    )?;
                    write_line(&mut stream, RESP_ERROR)?;
                }
            }
            Request::Pass(password) => {
                if session.state != SessionState::AuthRequired {
                    append_log(
                        &logs_dir.join("exception.log"),
                        &format!("{peer} PASS in invalid state"),
                    )?;
                    write_line(&mut stream, RESP_ERROR)?;
                    continue;
                }

                let Some(user_id) = session.current_user.as_ref() else {
                    write_line(&mut stream, RESP_ERROR)?;
                    continue;
                };

                let ok = users.get(user_id).map(|pin| pin == &password).unwrap_or(false);
                if ok {
                    session.state = SessionState::LoggedIn;
                    write_line(&mut stream, RESP_OK)?;
                } else {
                    append_log(
                        &logs_dir.join("exception.log"),
                        &format!("{peer} password failed for user {user_id}"),
                    )?;
                    write_line(&mut stream, RESP_ERROR)?;
                }
            }
            Request::Bala => {
                if session.state != SessionState::LoggedIn {
                    append_log(
                        &logs_dir.join("exception.log"),
                        &format!("{peer} BALA in invalid state"),
                    )?;
                    write_line(&mut stream, RESP_ERROR)?;
                    continue;
                }

                let user_id = session.current_user.as_ref().expect("logged in user");
                let amount = {
                    let balances_guard = balances.lock().expect("balances mutex poisoned");
                    balances_guard.get(user_id).copied()
                };

                if let Some(balance) = amount {
                    write_line(&mut stream, &format_amount_response(balance))?;
                } else {
                    append_log(
                        &logs_dir.join("exception.log"),
                        &format!("{peer} missing balance for user {user_id}"),
                    )?;
                    write_line(&mut stream, RESP_ERROR)?;
                }
            }
            Request::Wdra(amount) => {
                if session.state != SessionState::LoggedIn {
                    append_log(
                        &logs_dir.join("exception.log"),
                        &format!("{peer} WDRA in invalid state"),
                    )?;
                    write_line(&mut stream, RESP_ERROR)?;
                    continue;
                }

                if !amount.is_finite() || amount <= 0.0 {
                    append_log(
                        &logs_dir.join("exception.log"),
                        &format!("{peer} invalid withdraw amount: {amount}"),
                    )?;
                    write_line(&mut stream, RESP_ERROR)?;
                    continue;
                }

                let user_id = session.current_user.as_ref().expect("logged in user").clone();
                let mut balances_guard = balances.lock().expect("balances mutex poisoned");
                let Some(balance) = balances_guard.get_mut(&user_id) else {
                    append_log(
                        &logs_dir.join("exception.log"),
                        &format!("{peer} missing balance for user {user_id}"),
                    )?;
                    write_line(&mut stream, RESP_ERROR)?;
                    continue;
                };

                if *balance >= amount {
                    let before = *balance;
                    *balance -= amount;
                    let after = *balance;

                    save_balances(&balances_path, &balances_guard)?;
                    append_log(
                        &logs_dir.join("withdraw.log"),
                        &format!(
                            "{peer} user={user_id} withdraw={amount:.2} before={before:.2} after={after:.2}"
                        ),
                    )?;
                    write_line(&mut stream, RESP_OK)?;
                } else {
                    append_log(
                        &logs_dir.join("exception.log"),
                        &format!(
                            "{peer} insufficient funds user={user_id} request={amount:.2} balance={:.2}",
                            *balance
                        ),
                    )?;
                    write_line(&mut stream, RESP_ERROR)?;
                }
            }
            Request::Quit => {
                session.state = SessionState::Init;
                session.current_user = None;
                write_line(&mut stream, RESP_BYE)?;
                append_log(&logs_dir.join("server.log"), &format!("session bye: {peer}"))?;
                break;
            }
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let port = parse_port_arg(std::env::args().nth(1));
    let bind_addr = format!("0.0.0.0:{port}");

    let (users_path, balances_path, logs_dir) = resolve_paths()?;
    let users = Arc::new(load_users(&users_path)?);
    let balances = Arc::new(Mutex::new(load_balances(&balances_path)?));

    let listener = TcpListener::bind(&bind_addr)?;
    append_log(
        &logs_dir.join("server.log"),
        &format!("server listening on {bind_addr}"),
    )?;
    println!("Rust ATM server started on {bind_addr}");

    for incoming in listener.incoming() {
        match incoming {
            Ok(stream) => {
                let users = Arc::clone(&users);
                let balances = Arc::clone(&balances);
                let balances_path = balances_path.clone();
                let logs_dir = logs_dir.clone();

                thread::spawn(move || {
                    if let Err(error) =
                        handle_client(stream, users, balances, balances_path, logs_dir.clone())
                    {
                        let _ = append_log(
                            &logs_dir.join("exception.log"),
                            &format!("client handler crashed: {error}"),
                        );
                    }
                });
            }
            Err(error) => {
                append_log(
                    &logs_dir.join("exception.log"),
                    &format!("accept error: {error}"),
                )?;
            }
        }
    }

    Ok(())
}
