use bevy_editor_pls::egui::{Color32, RichText, ScrollArea, Ui};
use cansi::v3::categorise_text;
use crossbeam_channel::{Receiver, Sender};
use std::io::Read;
use std::io::Write;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub fn rich_text_display_multiline(ui: &mut Ui, rich_texts: &[RichText]) {
    let mut amount = 0;
    for _ in 0..rich_texts.len() {
        let mut reached_the_end = true;
        ui.horizontal(|ui| {
            for (i, rich_text) in rich_texts.iter().enumerate() {
                if i <= amount {
                    continue;
                }
                ui.label(rich_text.clone());
                if rich_text.text().contains('\n') {
                    amount = i;
                    reached_the_end = false;
                    break;
                }
            }
        });
        if reached_the_end {
            break;
        }
    }
}

pub fn from_cansi_to_egui_color(color: cansi::Color) -> Color32 {
    match color {
        cansi::Color::Black => Color32::BLACK,
        cansi::Color::Red => Color32::RED,
        cansi::Color::Green => Color32::GREEN,
        cansi::Color::Yellow => Color32::YELLOW,
        cansi::Color::Blue => Color32::BLUE,
        cansi::Color::Magenta => Color32::from_rgb(255, 0, 255),
        cansi::Color::Cyan => Color32::LIGHT_BLUE,
        cansi::Color::White => Color32::WHITE,
        cansi::Color::BrightBlack => Color32::BLACK,
        cansi::Color::BrightRed => Color32::LIGHT_RED,
        cansi::Color::BrightGreen => Color32::LIGHT_GREEN,
        cansi::Color::BrightYellow => Color32::LIGHT_YELLOW,
        cansi::Color::BrightBlue => Color32::LIGHT_BLUE,
        cansi::Color::BrightMagenta => Color32::from_rgb(255, 30, 255),
        cansi::Color::BrightCyan => Color32::LIGHT_BLUE,
        cansi::Color::BrightWhite => Color32::WHITE,
    }
}

pub fn rich_text_vec(string: &str) -> Vec<RichText> {
    let mut rich_texts = vec![];
    let text = categorise_text(string);
    for t in text.iter() {
        let mut rich_text = RichText::new(t.text);
        if let Some(fg) = t.fg {
            rich_text = rich_text.color(from_cansi_to_egui_color(fg));
        }
        if let Some(italics) = t.italic {
            if italics {
                rich_text = rich_text.italics();
            }
        }
        if let Some(bg) = t.bg {
            rich_text = rich_text.background_color(from_cansi_to_egui_color(bg));
        }
        rich_texts.push(rich_text);
    }
    rich_texts
}

fn create_tcp_listener(port: u16) -> TcpStream {
    match TcpListener::bind(format!("localhost:{}", port)) {
        Ok(listener) => {
            println!("bound to: {port}");
            let (listener, _) = listener.accept().unwrap();
            return listener;
        },
        Err(err) => {
            panic!("{err}")
        }
    }
}

fn create_tcp_stream(port: u16) -> TcpStream {
    for i in 0..50 {
        match TcpStream::connect(format!("localhost:{port}")) {
            Ok(stream) => {
                println!("connected to: {port}");
                return stream;
            },
            Err(err) => {
                if i >= 10 {
                    eprintln!("{err}");
                }
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
    panic!("ran out of iterations no tcp connection to game working");
}

pub fn setup_streams() -> (Receiver<u8>, Sender<u8>) {
    let stream = create_tcp_stream(std::env::var("RX_PORT").unwrap().parse().unwrap());
    let mut listener = create_tcp_listener(std::env::var("TX_PORT").unwrap().parse().unwrap());


    let (terminal_output_tx, mut terminal_output_rx) = crossbeam_channel::unbounded();

    let (quit_reason_tx, quit_reason_rx) = crossbeam_channel::unbounded();

    thread::spawn(move || {
        for b in stream.bytes() {
            if let Ok(b) = b {
                terminal_output_tx.send(b).unwrap();
            }
        }
    });

    thread::spawn(move || {
        let b = quit_reason_rx.recv().unwrap();
        listener.write(&[b]).expect("TODO: panic message");
        listener.flush().expect("TODO: panic message");
    });
    (terminal_output_rx, quit_reason_tx)
}

pub fn display_terminal(
    terminal_string: &mut String,
    terminal_output: Receiver<u8>,
    ui: &mut Ui,
    scroll_to_bottom: bool,
) {
    for _ in 0..terminal_output.len() + 20 {
        for byte in terminal_output.try_recv() {
            //print!("{}", char::from(byte));
            terminal_string.push(char::from(byte));
        }
    }
    ScrollArea::new(true).show(ui, |ui| {
        rich_text_display_multiline(ui, &rich_text_vec(terminal_string.as_str()));
        let rect = ui.label("").rect;
        if scroll_to_bottom {
            ui.scroll_to_rect(rect, None);
        }
    });
}
