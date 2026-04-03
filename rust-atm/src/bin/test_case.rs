use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;

fn send_and_recv(stream: &mut TcpStream, reader: &mut BufReader<TcpStream>, req: &str) -> io::Result<String> {
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
    let mut stream = TcpStream::connect(&addr)?;
    let mut reader = BufReader::new(stream.try_clone()?);

    let steps = vec![
        format!("HELO {user}"),
        format!("PASS {pass}"),
        "BALA".to_string(),
        format!("WDRA {amount}"),
        "BALA".to_string(),
        "BYE".to_string(),
    ];

    for req in steps {
        let resp = send_and_recv(&mut stream, &mut reader, &req)?;
        println!(">> {req}");
        println!("<< {resp}");
    }

    println!("\n[CASE2] wrong password");
    let mut stream2 = TcpStream::connect(&addr)?;
    let mut reader2 = BufReader::new(stream2.try_clone()?);

    for req in [
        format!("HELO {user}"),
        "PASS wrong_password".to_string(),
        "BYE".to_string(),
    ] {
        let resp = send_and_recv(&mut stream2, &mut reader2, &req)?;
        println!(">> {req}");
        println!("<< {resp}");
    }

    println!("\n[CASE3] insufficient funds");
    let mut stream3 = TcpStream::connect(&addr)?;
    let mut reader3 = BufReader::new(stream3.try_clone()?);

    for req in [
        format!("HELO {user}"),
        format!("PASS {pass}"),
        "WDRA 9999999".to_string(),
        "BYE".to_string(),
    ] {
        let resp = send_and_recv(&mut stream3, &mut reader3, &req)?;
        println!(">> {req}");
        println!("<< {resp}");
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let host = args.get(1).cloned().unwrap_or_else(|| "127.0.0.1".to_string());
    let port = args
        .get(2)
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(2525);
    let user = args.get(3).cloned().unwrap_or_else(|| "10001".to_string());
    let pass = args.get(4).cloned().unwrap_or_else(|| "111111".to_string());
    let amount = args
        .get(5)
        .and_then(|a| a.parse::<f64>().ok())
        .unwrap_or(100.0);

    if let Err(e) = run_case(&host, port, &user, &pass, amount) {
        eprintln!("test case failed: {e}");
        std::process::exit(1);
    }
}
