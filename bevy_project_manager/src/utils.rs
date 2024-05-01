use cansi::v3::categorise_text;
use crossbeam_channel::Receiver;
use egui::{Color32, RichText, ScrollArea, Ui};
use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

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

pub fn command_channels(mut command: Command) -> (Receiver<u8>, Child) {
    command.stderr(Stdio::piped());
    command.stdout(Stdio::piped());
    let mut child = command.spawn().unwrap();
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let (tx, rx) = crossbeam_channel::unbounded();

    let tx2 = tx.clone();
    thread::spawn(move || {
        for b in stdout.bytes() {
            if let Ok(b) = b {
                tx2.send(b).unwrap()
            }
        }
    });
    thread::spawn(move || {
        for b in stderr.bytes() {
            if let Ok(b) = b {
                tx.send(b).unwrap()
            }
        }
    });

    (rx, child)
}

pub fn from_cansi_to_egui_color(color: cansi::Color) -> egui::Color32 {
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
        let mut rich_text = egui::RichText::new(t.text);
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

pub fn display_terminal(terminal_string: Arc<Mutex<String>>, rx: Receiver<u8>, ui: &mut Ui) {
    for _ in 0..rx.len() + 20 {
        for byte in rx.recv() {
            print!("{}", char::from(byte));
            terminal_string.lock().unwrap().push(char::from(byte));
        }
    }
    ScrollArea::new(true).show(ui, |ui| {
        rich_text_display_multiline(ui, &rich_text_vec(terminal_string.lock().unwrap().as_str()));
        let rect = ui.label("").rect;
        ui.scroll_to_rect(rect, None);
    });
}
