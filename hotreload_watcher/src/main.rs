// I attempted to style the code in this file based on the
// "Why You Shouldn't Nest Your Code" YouTube Video.
// I like experimenting. See how it turns out for yourself.

use std::env::args;
use crossbeam_channel::Receiver;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

fn main() {
    // only ui only does ui.
    let mut only_ui = false;
    let mut position = None;
    for (i, arg) in args().enumerate() {
        if i == 1 {
            position.replace(arg);
        }
    }
    let location_to_run = position.unwrap();
    loop {
        let (tx_port, rx_port) = ( portpicker::pick_unused_port_range(1026..10_000).expect("No free tcp port"), portpicker::pick_unused_port_range(1026..10_000).expect("No free tcp port"));
        let child_command = create_child_process(only_ui, location_to_run.clone(), rx_port, tx_port);
        let (rx, child) = spawn_child_with_std_out_err_channel(child_command);
        let tx_editor = create_tcp_listener(tx_port);
        let rx_editor = create_tcp_stream(rx_port);
        if let QuitType::Clean = run_child_loop(&mut only_ui, tx_editor, rx_editor, rx, child) {
            return;
        }
    }
}

fn run_child_loop(
    only_ui: &mut bool,
    mut tx_editor: TcpStream,
    mut rx_editor: TcpStream,
    rx: Receiver<u8>,
    mut child: Child,
) -> QuitType {
    loop {
        while let Ok(byte) = rx.recv() {
            if let Err(_) = tx_editor.write(&[byte]) {
                break;
            }
        }
        let Ok(maybe_status) = child.try_wait() else {
            break;
        };
        let Some(status) = maybe_status else { break };
        return if status.success() {
            QuitType::Clean
        } else {
            *only_ui = true;
            QuitType::Unclean
        };
    }
    let mut turn_off_only_ui = [0];
    rx_editor.read(&mut turn_off_only_ui).unwrap();
    if turn_off_only_ui[0] == 1 {
        *only_ui = false;
    }
    return QuitType::Unclean;
}

enum QuitType {
    Clean,
    Unclean,
}

fn create_tcp_listener(port: u16) -> TcpStream {
    match TcpListener::bind(format!("127.0.0.0:{}", port)) {
        Ok(listener) => {
            println!("bound to: {port}");
            let (tx_editor, _) = listener.accept().unwrap();
            tx_editor
        },
        Err(err) => {
            panic!("{err}")
        }
    }
}

fn create_tcp_stream(port: u16) -> TcpStream {
    for i in 0..500 {
        match TcpStream::connect(format!("127.0.0.0:{port}")) {
            Ok(stream) => {
                println!("connected to: {port}");
                return stream;
            },
            Err(err) => {
                if i >= 50 {
                    eprintln!("{err}");
                }
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
    panic!("ran out of iterations no tcp connection to game working");
}

fn create_child_process(only_ui: bool, location_to_run: String, rx_port: u16, tx_port: u16) -> Command {
    let mut command = std::process::Command::new("dexterous_developer_cli");
    command.arg("run");
    command.env("TX_PORT", rx_port.to_string());
    command.env("RX_PORT", tx_port.to_string());
    command.current_dir(location_to_run);

    if only_ui {
        command.env("ONLY_UI", "true");
    }
    println!("command created");
    command
}

pub fn spawn_child_with_std_out_err_channel(mut command: Command) -> (Receiver<u8>, Child) {
    command.stderr(Stdio::piped());
    command.stdout(Stdio::piped());
    let mut child = command.spawn().unwrap();
    let stdout = child.stdout.take().unwrap().bytes();
    let stderr = child.stderr.take().unwrap().bytes();
    let (tx, rx) = crossbeam_channel::unbounded();
    let tx2 = tx.clone();

    thread::spawn(move || {
        for b in stdout {
            if let Ok(b) = b {
                print!("{}", char::from(b));
                match tx.send(b) {
                    Err(err) => eprintln!("child_std_thread error: {err}"),
                    _ => (),
                }
            }
        }
    });
    thread::spawn(move || {
        for b in stderr {
            if let Ok(b) = b {
                print!("{}", char::from(b));
                match tx2.send(b) {
                    Err(err) => eprintln!("child_std_thread error: {err}"),
                    _ => (),
                }
            }
        }
    });

    (rx, child)
}
