use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

const DEFAULT_HOST: &str = "172.19.153.48";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);

fn connect_with_timeout(addr: &str) -> io::Result<TcpStream> {
    let socket_addr = addr
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid address"))?;
    TcpStream::connect_timeout(&socket_addr, CONNECT_TIMEOUT)
}

fn send_and_recv(
    stream: &mut TcpStream,
    reader: &mut BufReader<TcpStream>,
    req: &str,
) -> io::Result<String> {
    stream.write_all(req.as_bytes())?;
    stream.write_all(b"\n")?;
    stream.flush()?;

    let mut resp = String::new();
    reader.read_line(&mut resp)?;
    Ok(resp.trim().to_string())
}

fn run_case(host: &str, port: u16, user: &str, pass: &str, amount: f64) -> io::Result<()> {
    let addr = format!("{host}:{port}");

    println!("[CASE1] normal flow against {addr}");
    let mut stream = connect_with_timeout(&addr)?;
    let mut reader = BufReader::new(stream.try_clone()?);

    let steps = vec![
        format!("HELO {user}"),
        format!("PASS {pass}"),
        "BALA".to_string(),
        format!("WDRA {amount}"),
        "BALA".to_string(),
        "QUIT".to_string(),
    ];

    for req in steps {
        let resp = send_and_recv(&mut stream, &mut reader, &req)?;
        println!(">> {req}");
        println!("<< {resp}");
    }

    println!("\n[CASE2] wrong password");
    let mut stream2 = connect_with_timeout(&addr)?;
    let mut reader2 = BufReader::new(stream2.try_clone()?);

    for req in [
        format!("HELO {user}"),
        "PASS wrong_password".to_string(),
        "QUIT".to_string(),
    ] {
        let resp = send_and_recv(&mut stream2, &mut reader2, &req)?;
        println!(">> {req}");
        println!("<< {resp}");
    }

    println!("\n[CASE3] insufficient funds");
    let mut stream3 = connect_with_timeout(&addr)?;
    let mut reader3 = BufReader::new(stream3.try_clone()?);

    for req in [
        format!("HELO {user}"),
        format!("PASS {pass}"),
        "WDRA 9999999".to_string(),
        "QUIT".to_string(),
    ] {
        let resp = send_and_recv(&mut stream3, &mut reader3, &req)?;
        println!(">> {req}");
        println!("<< {resp}");
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let host = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| DEFAULT_HOST.to_string());
    let port = args
        .get(2)
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(2525);
    let user = args
        .get(3)
        .cloned()
        .unwrap_or_else(|| "100001".to_string());
    let pass = args
        .get(4)
        .cloned()
        .unwrap_or_else(|| "1234".to_string());
    let amount = args
        .get(5)
        .and_then(|a| a.parse::<f64>().ok())
        .unwrap_or(100.0);

    if let Err(error) = run_case(&host, port, &user, &pass, amount) {
        eprintln!("test case failed: {error}");
        std::process::exit(1);
    }
}
