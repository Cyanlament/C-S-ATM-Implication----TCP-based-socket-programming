use rust_atm::{
    append_log, format_amount_response, load_accounts, parse_request, save_accounts, Request,
    RESP_AUTH_REQUIRED, RESP_BYE, RESP_ERROR, RESP_OK,
};
use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug)]
struct Session {
    current_user: Option<String>,
    authenticated: bool,
}

impl Session {
    fn new() -> Self {
        Self {
            current_user: None,
            authenticated: false,
        }
    }
}

fn write_line(stream: &mut TcpStream, line: &str) -> io::Result<()> {
    stream.write_all(line.as_bytes())?;
    stream.write_all(b"\n")?;
    stream.flush()
}

fn handle_client(
    mut stream: TcpStream,
    db: Arc<Mutex<rust_atm::AccountsDb>>,
    data_path: PathBuf,
    logs_dir: PathBuf,
) -> io::Result<()> {
    let peer = stream
        .peer_addr()
        .map(|a| a.to_string())
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

        let req = parse_request(raw);
        if req.is_none() {
            // 不认识的命令，先记日志再回 401，主打有据可查。
            append_log(
                &logs_dir.join("exception.log"),
                &format!("{peer} invalid request: {raw}"),
            )?;
            write_line(&mut stream, RESP_ERROR)?;
            continue;
        }

        match req.expect("checked is_some") {
            Request::Helo(user_id) => {
                let exists = {
                    let db_guard = db.lock().expect("accounts db mutex poisoned");
                    db_guard.contains_key(&user_id)
                };

                if exists {
                    session.current_user = Some(user_id);
                    session.authenticated = false;
                    write_line(&mut stream, RESP_AUTH_REQUIRED)?;
                } else {
                    append_log(
                        &logs_dir.join("exception.log"),
                        &format!("{peer} unknown user id"),
                    )?;
                    write_line(&mut stream, RESP_ERROR)?;
                }
            }
            Request::Pass(password) => {
                let Some(user_id) = &session.current_user else {
                    append_log(
                        &logs_dir.join("exception.log"),
                        &format!("{peer} PASS before HELO"),
                    )?;
                    write_line(&mut stream, RESP_ERROR)?;
                    continue;
                };

                let ok = {
                    let db_guard = db.lock().expect("accounts db mutex poisoned");
                    db_guard
                        .get(user_id)
                        .map(|acc| acc.password == password)
                        .unwrap_or(false)
                };

                if ok {
                    session.authenticated = true;
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
                if !session.authenticated {
                    if session.current_user.is_some() {
                        write_line(&mut stream, RESP_AUTH_REQUIRED)?;
                    } else {
                        write_line(&mut stream, RESP_ERROR)?;
                    }
                    continue;
                }

                let user_id = session.current_user.as_ref().expect("authed has user id");
                let balance = {
                    let db_guard = db.lock().expect("accounts db mutex poisoned");
                    db_guard.get(user_id).map(|acc| acc.balance).unwrap_or(-1.0)
                };

                if balance < 0.0 {
                    append_log(
                        &logs_dir.join("exception.log"),
                        &format!("{peer} user not found while BALA: {user_id}"),
                    )?;
                    write_line(&mut stream, RESP_ERROR)?;
                } else {
                    write_line(&mut stream, &format_amount_response(balance))?;
                }
            }
            Request::Wdra(amount) => {
                if amount <= 0.0 {
                    append_log(
                        &logs_dir.join("exception.log"),
                        &format!("{peer} invalid withdraw amount: {amount}"),
                    )?;
                    write_line(&mut stream, RESP_ERROR)?;
                    continue;
                }

                if !session.authenticated {
                    if session.current_user.is_some() {
                        write_line(&mut stream, RESP_AUTH_REQUIRED)?;
                    } else {
                        write_line(&mut stream, RESP_ERROR)?;
                    }
                    continue;
                }

                let user_id = session
                    .current_user
                    .clone()
                    .expect("authenticated session should have user id");

                let mut db_guard = db.lock().expect("accounts db mutex poisoned");
                let Some(account) = db_guard.get_mut(&user_id) else {
                    append_log(
                        &logs_dir.join("exception.log"),
                        &format!("{peer} user not found while WDRA: {user_id}"),
                    )?;
                    write_line(&mut stream, RESP_ERROR)?;
                    continue;
                };

                if account.balance >= amount {
                    let before = account.balance;
                    account.balance -= amount;
                    let after = account.balance;

                    // 取款成功后立刻落盘，防止“余额失忆”。
                    save_accounts(&data_path, &db_guard)?;
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
                            account.balance
                        ),
                    )?;
                    write_line(&mut stream, RESP_ERROR)?;
                }
            }
            Request::Bye => {
                write_line(&mut stream, RESP_BYE)?;
                append_log(&logs_dir.join("server.log"), &format!("session bye: {peer}"))?;
                break;
            }
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let bind_addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "0.0.0.0:2525".to_string());

    let data_path = PathBuf::from("data/accounts.json");
    let logs_dir = PathBuf::from("logs");

    let db = load_accounts(&data_path)?;
    let shared_db = Arc::new(Mutex::new(db));

    let listener = TcpListener::bind(&bind_addr)?;
    append_log(
        &logs_dir.join("server.log"),
        &format!("server listening on {bind_addr}"),
    )?;

    for incoming in listener.incoming() {
        match incoming {
            Ok(stream) => {
                let db = Arc::clone(&shared_db);
                let data_path = data_path.clone();
                let logs_dir = logs_dir.clone();

                thread::spawn(move || {
                    // 每个客户端一条线程，大家排队办业务。
                    if let Err(e) = handle_client(stream, db, data_path, logs_dir.clone()) {
                        let _ = append_log(
                            &logs_dir.join("exception.log"),
                            &format!("client handler crashed: {e}"),
                        );
                    }
                });
            }
            Err(e) => {
                append_log(
                    &logs_dir.join("exception.log"),
                    &format!("accept error: {e}"),
                )?;
            }
        }
    }

    Ok(())
}
