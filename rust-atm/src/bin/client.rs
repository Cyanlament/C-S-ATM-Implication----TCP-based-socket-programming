use eframe::egui::{self, Color32, RichText, Stroke};
use rust_atm::RESP_BYE;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

const DEFAULT_HOST: &str = "172.19.153.48";
const DEFAULT_PORT: &str = "2525";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);

struct Connection {
    writer: TcpStream,
    reader: BufReader<TcpStream>,
}

impl Connection {
    fn connect(addr: &str) -> io::Result<Self> {
        let socket_addr = addr
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid address"))?;
        let writer = TcpStream::connect_timeout(&socket_addr, CONNECT_TIMEOUT)?;
        writer.set_read_timeout(Some(Duration::from_secs(5)))?;
        writer.set_write_timeout(Some(Duration::from_secs(5)))?;
        let reader = BufReader::new(writer.try_clone()?);
        Ok(Self { writer, reader })
    }

    fn send_line(&mut self, req: &str) -> io::Result<String> {
        self.writer.write_all(req.as_bytes())?;
        self.writer.write_all(b"\n")?;
        self.writer.flush()?;

        let mut line = String::new();
        let n = self.reader.read_line(&mut line)?;
        if n == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "server closed connection",
            ));
        }

        Ok(line.trim().to_string())
    }
}

struct AtmGui {
    host: String,
    port: String,
    user_id: String,
    password: String,
    withdraw_amount: String,
    status: String,
    transcript: String,
    conn: Option<Connection>,
}

fn friendly_error(error: &io::Error) -> String {
    match error.kind() {
        io::ErrorKind::ConnectionRefused => {
            "Connection refused. Start the server and check the port.".to_string()
        }
        io::ErrorKind::TimedOut => {
            "Connection timeout. Check the LAN IP, firewall, and Wi-Fi network.".to_string()
        }
        io::ErrorKind::NotFound | io::ErrorKind::AddrNotAvailable => {
            "Address is not reachable on this machine.".to_string()
        }
        io::ErrorKind::InvalidInput => "Invalid host or port.".to_string(),
        io::ErrorKind::UnexpectedEof => "Server closed the connection.".to_string(),
        io::ErrorKind::WouldBlock => "Network operation timed out.".to_string(),
        _ => format!("Network error ({:?})", error.kind()),
    }
}

fn client_defaults_from_args() -> (String, String) {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let host = args
        .first()
        .cloned()
        .unwrap_or_else(|| DEFAULT_HOST.to_string());
    let port = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| DEFAULT_PORT.to_string());
    (host, port)
}

impl Default for AtmGui {
    fn default() -> Self {
        let (host, port) = client_defaults_from_args();
        Self {
            host,
            port,
            user_id: "100001".to_string(),
            password: "1234".to_string(),
            withdraw_amount: "100".to_string(),
            status: "Disconnected".to_string(),
            transcript: String::new(),
            conn: None,
        }
    }
}

impl AtmGui {
    fn append_transcript(&mut self, line: &str) {
        self.transcript.push_str(line);
        self.transcript.push('\n');
    }

    fn server_addr(&self) -> String {
        format!("{}:{}", self.host.trim(), self.port.trim())
    }

    fn connect(&mut self) {
        let addr = self.server_addr();
        match Connection::connect(&addr) {
            Ok(c) => {
                // 连上就开工，今天也是认真取钱的一天。
                self.conn = Some(c);
                self.status = format!("Connected to {addr}");
                self.append_transcript(&format!("[SYS] connected: {addr}"));
            }
            Err(e) => {
                let message = friendly_error(&e);
                self.status = format!("Connect failed: {message}");
                self.append_transcript(&format!("[ERR] connect failed: {message}"));
            }
        }
    }

    fn disconnect(&mut self) {
        self.conn = None;
        self.status = "Disconnected".to_string();
        self.append_transcript("[SYS] disconnected");
    }

    fn send_request(&mut self, request: String) {
        if self.conn.is_none() {
            self.status = "Not connected".to_string();
            self.append_transcript("[ERR] not connected");
            return;
        }

        self.append_transcript(&format!(">> {request}"));
        let send_result = {
            let conn = self.conn.as_mut().expect("checked is_some");
            conn.send_line(&request)
        };

        match send_result {
            Ok(resp) => {
                self.append_transcript(&format!("<< {resp}"));
                self.status = format!("Server: {resp}");

                if resp == RESP_BYE {
                    self.conn = None;
                }
            }
            Err(e) => {
                let message = friendly_error(&e);
                self.status = format!("Network error: {message}");
                self.append_transcript(&format!("[ERR] network error: {message}"));
                self.conn = None;
            }
        }
    }

    fn run_demo_flow(&mut self) {
        // 一键演示，适合课堂展示。
        self.send_request(format!("HELO {}", self.user_id.trim()));
        self.send_request(format!("PASS {}", self.password.trim()));
        self.send_request("BALA".to_string());
        self.send_request(format!("WDRA {}", self.withdraw_amount.trim()));
        self.send_request("BALA".to_string());
        self.send_request("QUIT".to_string());
    }

    fn status_style(&self) -> (Color32, &'static str) {
        let lower = self.status.to_lowercase();
        if self.status.contains("401") || lower.contains("error") || lower.contains("failed") {
            (Color32::from_rgb(185, 28, 28), "Status: ERROR")
        } else if self.status.contains("500") {
            (Color32::from_rgb(124, 45, 18), "Status: AUTH REQUIRE")
        } else if self.status.contains("525")
            || self.status.contains("AMNT:")
            || self.status.contains("BYE")
            || lower.starts_with("connected")
        {
            (Color32::from_rgb(21, 128, 61), "Status: OK")
        } else {
            (Color32::from_rgb(51, 65, 85), "Status")
        }
    }

    fn draw_controls(&mut self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style())
            .fill(Color32::WHITE)
            .stroke(Stroke::new(1.0, Color32::from_rgb(203, 213, 225)))
            .show(ui, |ui| {
                ui.heading("Control Panel");
                ui.label("Protocol: HELO / PASS / BALA / WDRA / QUIT");
                ui.separator();

                egui::Grid::new("conn_grid")
                    .num_columns(4)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Host");
                        ui.add_sized([210.0, 26.0], egui::TextEdit::singleline(&mut self.host));
                        ui.label("Port");
                        ui.add_sized([120.0, 26.0], egui::TextEdit::singleline(&mut self.port));
                        ui.end_row();

                        ui.label("User ID");
                        ui.add_sized([210.0, 26.0], egui::TextEdit::singleline(&mut self.user_id));
                        ui.label("Password");
                        ui.add_sized(
                            [120.0, 26.0],
                            egui::TextEdit::singleline(&mut self.password).password(true),
                        );
                        ui.end_row();

                        ui.label("Withdraw");
                        ui.add_sized(
                            [210.0, 26.0],
                            egui::TextEdit::singleline(&mut self.withdraw_amount),
                        );
                        ui.label("Tips");
                        ui.label("Use 172.19.153.48 for LAN");
                        ui.end_row();
                    });

                ui.add_space(6.0);
                ui.horizontal_wrapped(|ui| {
                    if ui.button("Connect").clicked() {
                        self.connect();
                    }
                    if ui.button("Disconnect").clicked() {
                        self.disconnect();
                    }
                    if ui.button("Send HELO").clicked() {
                        self.send_request(format!("HELO {}", self.user_id.trim()));
                    }
                    if ui.button("Send PASS").clicked() {
                        self.send_request(format!("PASS {}", self.password.trim()));
                    }
                });

                ui.horizontal_wrapped(|ui| {
                    if ui.button("Query BALA").clicked() {
                        self.send_request("BALA".to_string());
                    }
                    if ui.button("Send WDRA").clicked() {
                        self.send_request(format!("WDRA {}", self.withdraw_amount.trim()));
                    }
                    if ui.button("Send QUIT").clicked() {
                        self.send_request("QUIT".to_string());
                    }
                    if ui.button("Run Demo Flow").clicked() {
                        self.run_demo_flow();
                    }
                });

                ui.separator();
                let (status_color, status_title) = self.status_style();
                ui.label(
                    RichText::new(format!("{status_title}  |  {}", self.status))
                        .color(status_color)
                        .strong(),
                );
            });
    }

    fn draw_logs(&mut self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style())
            .fill(Color32::WHITE)
            .stroke(Stroke::new(1.0, Color32::from_rgb(203, 213, 225)))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Session Log");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Clear Log").clicked() {
                            self.transcript.clear();
                        }
                    });
                });
                ui.separator();
                ui.add_sized(
                    [ui.available_width(), 450.0],
                    egui::TextEdit::multiline(&mut self.transcript)
                        .font(egui::TextStyle::Monospace)
                        .interactive(false),
                );
                ui.label("Tip: lines starting with >> / << are protocol request/response.");
            });
    }
}

impl eframe::App for AtmGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut visuals = egui::Visuals::light();
        visuals.panel_fill = Color32::from_rgb(244, 248, 252);
        visuals.widgets.active.bg_fill = Color32::from_rgb(3, 105, 161);
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(2, 132, 199);
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(226, 232, 240);
        ctx.set_visuals(visuals);

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::group(ui.style())
                .fill(Color32::from_rgb(9, 69, 95))
                .stroke(Stroke::new(1.0, Color32::from_rgb(14, 116, 144)))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new("Rust ATM Client")
                            .size(28.0)
                            .color(Color32::from_rgb(240, 249, 255))
                            .strong(),
                    );
                    ui.label(
                        RichText::new("Socket Programming Demonstration Interface")
                            .color(Color32::from_rgb(186, 230, 253)),
                    );
                });

            ui.add_space(10.0);

            if ui.available_width() > 980.0 {
                ui.columns(2, |cols| {
                    self.draw_controls(&mut cols[0]);
                    self.draw_logs(&mut cols[1]);
                });
            } else {
                self.draw_controls(ui);
                ui.add_space(8.0);
                self.draw_logs(ui);
            }
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1180.0, 760.0])
            .with_min_inner_size([860.0, 620.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Rust ATM Client",
        options,
        Box::new(|_cc| Ok(Box::<AtmGui>::default())),
    )
}
