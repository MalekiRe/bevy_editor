use std::fs::DirEntry;
use std::path::PathBuf;
use bevy::prelude::World;
use bevy_editor_pls::egui;
use bevy_editor_pls::egui::{Align, Layout, RichText, Ui};
use bevy_editor_pls_core::editor_window::{EditorWindow, EditorWindowContext};
use egui_code_editor::{ColorTheme, Syntax};

pub struct CodeEditor;

pub struct CodeEditorState {
    code: String,
    selected_file: Option<PathBuf>,
}

impl Default for CodeEditor {
    fn default() -> Self {
        Self {}
    }
}
impl Default for CodeEditorState {
    fn default() -> Self {
        Self {
            code: "".to_string(),
            selected_file: None,
        }
    }
}

impl EditorWindow for CodeEditor {
    type State = CodeEditorState;
    const NAME: &'static str = "Code Editor";

    fn ui(world: &mut World, mut cx: EditorWindowContext, ui: &mut Ui) {
        let code = cx.state_mut::<CodeEditor>().unwrap();

        ui.horizontal_top(|ui| {
           let mut path_buf = None;
            ui.vertical(|ui| {
               let dir = std::fs::read_dir(std::env::current_dir().unwrap()).unwrap();
                for dir in dir.into_iter() {
                    if let Ok(dir) = dir {
                        if let Some(dir) = display_dir(ui, dir) {
                            path_buf.replace(dir);
                        }
                    }
                }
            });
            if let Some(buf) = path_buf {
                if let Some(old) = code.selected_file.clone() {
                    std::fs::write(old, &code.code).unwrap();
                }
                code.selected_file.replace(buf.clone());
                let reader = std::fs::read(buf).unwrap();
                code.code = String::from_utf8(reader).unwrap();
            }
            let mut control_key = false;
            ui.input(|input| {
                for event in &input.events {
                    match event {
                        egui::Event::Key{key, physical_key, pressed, repeat, modifiers } => {
                            if modifiers.ctrl && key.eq(&egui::Key::S) {
                                if let Some(old) = code.selected_file.clone() {
                                    std::fs::write(old, &code.code).unwrap();
                                }
                            }
                            if modifiers.ctrl {
                                control_key = true;
                            }
                        },
                        _ => {}
                    }
                }
            });

            if let Some(_) = code.selected_file.clone() {
                if ui.button("close").clicked() {
                    if let Some(old) = code.selected_file.clone() {
                        std::fs::write(old, &code.code).unwrap();
                    }
                    code.selected_file.take();
                }
                match control_key {
                    true => {
                        let mut temp = code.code.clone();
                        egui_code_editor::CodeEditor::default()
                            .id_source("code editor")
                            .with_rows(12)
                            .with_fontsize(14.0)
                            .with_theme(ColorTheme::GRUVBOX)
                            .with_syntax(Syntax::rust())
                            .with_numlines(true)
                            .show(ui, &mut temp);
                    },
                    false => {
                        egui_code_editor::CodeEditor::default()
                            .id_source("code editor")
                            .with_rows(12)
                            .with_fontsize(14.0)
                            .with_theme(ColorTheme::GRUVBOX)
                            .with_syntax(Syntax::rust())
                            .with_numlines(true)
                            .show(ui, &mut code.code);
                    }
                }
            }
        });
    }
}

fn display_dir(ui: &mut Ui, dir: DirEntry) -> Option<PathBuf> {
    if dir.file_type().unwrap().is_dir() {
        let mut path_buf = None;
        ui.collapsing(RichText::new(format!("{} {}", egui_phosphor::regular::FOLDER, dir.file_name().to_str().unwrap())), |ui| {
            for dir in  std::fs::read_dir(dir.path()).unwrap().into_iter() {
                if let Ok(dir) = dir {
                    if let Some(buf) = display_dir(ui, dir) {
                        path_buf.replace(buf);
                    }
                }
            }
        });
        path_buf
    } else {

        let icon = if dir.file_name().to_str().unwrap().ends_with(".rs") {
            egui_phosphor::regular::FILE_RS
        } else {
            egui_phosphor::regular::FILE
        };

        if ui.button(egui::RichText::new(format!("{} {}", icon, dir.file_name().to_str().unwrap()))).clicked() {
            Some(dir.path())
        } else {
            None
        }
    }
}