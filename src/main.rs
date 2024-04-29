use bevy::ecs::system::assert_system_does_not_conflict;
use crossbeam_channel::Receiver;
use directories::ProjectDirs;
use eframe::egui;
use eframe::egui::ScrollArea;
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use std::fs::DirEntry;
use std::io::{Error, Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Child, Command, ExitCode, ExitStatus, Stdio};
use std::str::FromStr;
use std::sync::mpsc::channel;
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    if !std::env::var("NORMAL_MODE").is_ok() {
        let has_rust = Command::new("cargo")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .is_ok();
        let mut has_dexterous_dev = false;
        if has_rust {
            has_dexterous_dev = Command::new("dexterous_developer_cli").spawn().is_ok();
        }
        eframe::run_native(
            "Native file dialogs and drag-and-drop files",
            eframe::NativeOptions::default(),
            Box::new(move |_cc| Box::new(MyApp::new(has_rust, has_dexterous_dev))),
        )
        .unwrap();
        return;
    }

    if std::env::var("IS_UWU").is_ok() {
        mylib::bevy_main();
    } else {
        let mut only_ui = false;
        loop {
            let mut child = match only_ui {
                true => std::process::Command::new("dexterous_developer_cli")
                    .env("IS_UWU", "true")
                    .env("ONLY_UI", "true")
                    .env("NORMAL_MODE", "true")
                    .arg("run")
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .unwrap(),
                false => std::process::Command::new("dexterous_developer_cli")
                    .env("IS_UWU", "true")
                    .env("NORMAL_MODE", "true")
                    .arg("run")
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .unwrap(),
            };
            println!("A");
            let mut stdout = child.stdout.take().unwrap();
            let mut stderr = child.stderr.take().unwrap();

            let (tx, rx) = crossbeam_channel::unbounded();

            let tx2 = tx.clone();
            thread::spawn(move || {
                for b in stdout.bytes() {
                    if let Ok(b) = b {
                        print!("{}", char::from(b));
                        tx2.send(b).unwrap();
                    }
                }
            });

            thread::spawn(move || {
                for b in stderr.bytes() {
                    if let Ok(b) = b {
                        print!("{}", char::from(b));
                        tx.send(b).unwrap();
                    }
                }
            });
            let listener = TcpListener::bind("127.0.0.0:8888").unwrap();

            let (mut stream, _) = listener.accept().unwrap();
            let mut break_da_loop = false;
            loop {
                if let Ok(val) = child.try_wait() {
                    match val {
                        None => {}
                        Some(status) => {
                            if status.success() {
                                break_da_loop = true;
                            } else {
                                if only_ui {
                                    //break_da_loop = true;
                                } else {
                                    only_ui = true;
                                }
                            }
                            break;
                        }
                    }
                }
                let mut break_inner = false;
                for byte in rx.recv() {
                    if byte == 195 {
                        println!("only ui is now false");
                        only_ui = false;
                        break_inner = true;
                    }
                    match stream.write(&[byte]) {
                        Ok(_) => (),
                        Err(_) => break,
                    }
                }
                if break_inner {
                    break;
                }
            }
            if break_da_loop {
                return;
            }
        }
    }
}

pub struct MyApp {
    has_rust: bool,
    has_dexterous_dev: bool,
    dexterous_install_child: Option<Child>,
    dexterous_install_output: String,
    dexterous_bytes: Option<Receiver<u8>>,
    database: PickleDb,
    projects: Vec<DirEntry>,
    create_new_project: bool,
    new_project_name: String,
    running_project: bool,
    running_bytes: Option<Receiver<u8>>,
    running_child: Option<Child>,
    running_output: String,
}

impl MyApp {
    pub fn new(has_rust: bool, has_dexterous_dev: bool) -> Self {
        let data_dir = ProjectDirs::from("com", "malek", "bevy_editor").unwrap();
        let data_dir = data_dir.data_dir();
        std::fs::create_dir_all(data_dir).unwrap();
        std::fs::create_dir_all(
            ProjectDirs::from("com", "malek", "bevy_editor")
                .unwrap()
                .data_dir()
                .with_file_name("projects"),
        )
        .unwrap();
        let path = data_dir.to_path_buf().with_file_name("bevy_editor.db");
        let pickel_db = match PickleDb::load(
            path.clone(),
            PickleDbDumpPolicy::AutoDump,
            SerializationMethod::Json,
        ) {
            Ok(db) => db,
            Err(_) => PickleDb::new(
                path,
                PickleDbDumpPolicy::AutoDump,
                SerializationMethod::Json,
            ),
        };
        let mut pos = PathBuf::from(
            ProjectDirs::from("com", "malek", "bevy_editor")
                .unwrap()
                .data_dir(),
        );
        pos.push("projects");
        let entries = std::fs::read_dir(pos).unwrap();
        let mut projects = vec![];
        for dir in entries {
            if let Ok(dir) = dir {
                projects.push(dir);
            }
        }
        Self {
            has_rust,
            has_dexterous_dev,
            dexterous_install_child: None,
            dexterous_install_output: "".to_string(),
            dexterous_bytes: None,
            database: pickel_db,
            projects,
            create_new_project: false,
            new_project_name: "".to_string(),
            running_project: false,
            running_bytes: None,
            running_child: None,
            running_output: "".to_string(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.running_project {
                ui.label("project is starting up");
                ScrollArea::new(true).show(ui, |ui| {
                    for _ in 0..300 {
                        for byte in self.running_bytes.as_mut().unwrap().try_recv() {
                            self.running_output.push(char::from(byte));
                        }
                    }
                    ui.label(&self.running_output);
                    let rect = ui.label("").rect;
                    ui.scroll_to_rect(rect, None);
                });
                return;
            }
            if !self.has_rust {
                ui.label("install rust");
                ui.hyperlink("https://www.rust-lang.org/learn/get-started");
            }
            if !self.has_dexterous_dev {
                if ui.button("install dexterous developer cli").clicked() {
                    let mut child = Command::new("cargo")
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .arg("install")
                        .arg("dexterous_developer_cli")
                        .spawn()
                        .unwrap();

                    let mut stdout = child.stdout.take().unwrap();
                    let mut stderr = child.stderr.take().unwrap();

                    let (tx, rx) = crossbeam_channel::unbounded();

                    let tx2 = tx.clone();
                    thread::spawn(move || {
                        for b in stdout.bytes() {
                            if let Ok(b) = b {
                                tx2.send(b).unwrap();
                            }
                        }
                    });

                    thread::spawn(move || {
                        for b in stderr.bytes() {
                            if let Ok(b) = b {
                                tx.send(b).unwrap();
                            }
                        }
                    });
                    self.dexterous_install_child.replace(child);
                    self.dexterous_bytes.replace(rx);
                }
                if let Some(child) = self.dexterous_install_child.as_mut() {
                    ui.label("installing");
                    ScrollArea::new(true).show(ui, |ui| {
                        for byte in self.dexterous_bytes.as_mut().unwrap().recv() {
                            self.dexterous_install_output.push(char::from(byte));
                        }
                        ui.label(&self.dexterous_install_output);
                    });
                    if child.try_wait().is_ok() {
                        self.has_dexterous_dev = true;
                    }
                }
            }
            if self.has_rust && self.has_dexterous_dev {
                ui.label("everything installed");
            } else {
                return;
            }
            if ui.button("reload").clicked() {
                let mut pos = PathBuf::from(
                    ProjectDirs::from("com", "malek", "bevy_editor")
                        .unwrap()
                        .data_dir(),
                );
                pos.push("projects");
                let entries = std::fs::read_dir(pos).unwrap();
                self.projects.clear();
                for dir in entries {
                    if let Ok(dir) = dir {
                        self.projects.push(dir);
                    }
                }
            }
            if ui
                .collapsing("projects", |ui| {
                    for project in &self.projects {
                        if ui.button(project.file_name().to_str().unwrap()).clicked() {
                            let mut child = Command::new("cargo")
                                .arg("run")
                                .stdout(Stdio::piped())
                                .stderr(Stdio::piped())
                                .env("NORMAL_MODE", "true")
                                .current_dir(project.path())
                                .spawn()
                                .unwrap();
                            let mut stdout = child.stdout.take().unwrap();
                            let mut stderr = child.stderr.take().unwrap();

                            let (tx, rx) = crossbeam_channel::unbounded();

                            let tx2 = tx.clone();
                            thread::spawn(move || {
                                for b in stdout.bytes() {
                                    if let Ok(b) = b {
                                        print!("{}", char::from(b));
                                        tx2.send(b).unwrap();
                                    }
                                }
                            });

                            thread::spawn(move || {
                                for b in stderr.bytes() {
                                    if let Ok(b) = b {
                                        print!("{}", char::from(b));
                                        tx.send(b).unwrap();
                                    }
                                }
                            });
                            self.running_child.replace(child);
                            self.running_bytes.replace(rx);
                            self.running_project = true;
                        }
                    }
                })
                .body_response
                .is_some_and(|a| a.clicked())
            {
                let mut pos = PathBuf::from(
                    ProjectDirs::from("com", "malek", "bevy_editor")
                        .unwrap()
                        .data_dir(),
                );
                pos.push("projects");
                let entries = std::fs::read_dir(pos).unwrap();
                self.projects.clear();
                for dir in entries {
                    if let Ok(dir) = dir {
                        self.projects.push(dir);
                    }
                }
            };
            if ui.button("create new project").clicked() {
                self.create_new_project = true;
            }
            if self.create_new_project {
                if ui.button("close").clicked() {
                    self.create_new_project = false;
                }
                ui.text_edit_singleline(&mut self.new_project_name);

                if ui.button("create").clicked() {
                    let mut new_project_position = PathBuf::from(
                        ProjectDirs::from("com", "malek", "bevy_editor")
                            .unwrap()
                            .data_dir(),
                    );

                    new_project_position.push("projects");
                    new_project_position.push(self.new_project_name.clone());

                    let mut assets_folder = new_project_position.clone();
                    assets_folder.push("assets");

                    let mut cargo_pos = new_project_position.clone();
                    cargo_pos.push("Cargo.toml");

                    let mut lock_pos = new_project_position.clone();
                    lock_pos.push("Cargo.lock");

                    let mut config_toml = new_project_position.clone();
                    config_toml.push(".cargo");
                    std::fs::create_dir_all(config_toml.clone()).unwrap();
                    config_toml.push("config.toml");

                    let mut src_pos = new_project_position.clone();
                    src_pos.push("src");

                    let mut main_pos = src_pos.clone();
                    main_pos.push("main.rs");

                    let mut lib_pos = src_pos.clone();
                    lib_pos.push("lib.rs");

                    std::fs::create_dir_all(new_project_position.clone()).unwrap();

                    let mut cargo_toml = std::fs::OpenOptions::new()
                        .write(true)
                        .read(true)
                        .create(true)
                        .open(cargo_pos)
                        .unwrap();
                    let mut cargo_lock = std::fs::OpenOptions::new()
                        .write(true)
                        .read(true)
                        .create(true)
                        .open(lock_pos)
                        .unwrap();
                    let mut config_toml = std::fs::OpenOptions::new()
                        .write(true)
                        .read(true)
                        .create(true)
                        .open(config_toml)
                        .unwrap();
                    std::fs::create_dir_all(assets_folder).unwrap();
                    std::fs::create_dir_all(src_pos).unwrap();
                    let mut main_rs = std::fs::OpenOptions::new()
                        .write(true)
                        .read(true)
                        .create(true)
                        .open(main_pos)
                        .unwrap();
                    let mut lib_rs = std::fs::OpenOptions::new()
                        .write(true)
                        .read(true)
                        .create(true)
                        .open(lib_pos)
                        .unwrap();

                    cargo_toml.write(CARGO_TOML).unwrap();
                    cargo_lock.write(CARGO_LOCK).unwrap();
                    main_rs.write(MAIN_RS).unwrap();
                    lib_rs.write(LIB_RS).unwrap();
                    config_toml.write(CONFIG_TOML).unwrap();
                    println!("creating: {:#?}", new_project_position.clone());
                }
            }
        });
    }
}

const MAIN_RS: &'static [u8] = include_bytes!("main.rs");
const LIB_RS: &'static [u8] = include_bytes!("lib.rs");
const CARGO_TOML: &'static [u8] = include_bytes!("../Cargo.toml");

const CARGO_LOCK: &'static [u8] = include_bytes!("../Cargo.lock");

const CONFIG_TOML: &'static [u8] = include_bytes!("../.cargo/config.toml");
