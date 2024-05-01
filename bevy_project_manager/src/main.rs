mod templates;
mod utils;

use crate::templates::Template;
use crossbeam_channel::Receiver;
use directories::ProjectDirs;
use eframe::emath::{Align, Vec2};
use eframe::{NativeOptions, WindowBuilder};
use egui::{include_image, Color32, ImageSource, Layout, Ui, WidgetText};
use egui_dock::{DockArea, DockState, Style, TabViewer};
use egui_dropdown::DropDownBox;
use egui_modal::Modal;
use std::fs::DirEntry;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};

fn main() {
    let mut path_buf = ProjectDirs::from("com", "malek", "bevy_editor")
        .unwrap()
        .data_dir()
        .to_path_buf();
    let mut cache_pos = ProjectDirs::from("com", "malek", "bevy_editor")
        .unwrap()
        .cache_dir()
        .to_path_buf();
    std::fs::create_dir_all(cache_pos.as_path()).unwrap();
    std::fs::create_dir_all(path_buf.as_path()).unwrap();




    cache_pos.push("hotreload_watcher");
    templates::Template::hot_reload_watcher()
        .build_template(cache_pos)
        .unwrap();

    let mut native_options = NativeOptions::default();
    native_options.viewport.icon.replace(Arc::new(
        eframe::icon_data::from_png_bytes(include_bytes!("../../assets/bevy_logo.png")).unwrap(),
    ));
    eframe::run_native(
        "Shitty Bevy Project Manager",
        native_options,
        Box::new(move |_cc| Box::new(MyApp::new())),
    )
    .unwrap();
}

fn get_hotreload_dir() -> PathBuf {
    let mut cache_pos = ProjectDirs::from("com", "malek", "bevy_editor")
        .unwrap()
        .cache_dir()
        .to_path_buf();
    cache_pos.push("hotreload_watcher");
    cache_pos
}

struct MyApp {
    tree: DockState<String>,
    app_states: AppStates,
}

pub enum AppStates {
    ProjectViewer(ProjectViewer),
    ProjectRunner(ProjectRunner),
    DexterousDevInstall(DexterousDevInstall)
}

pub struct DexterousDevInstall {
    child: Child,
    buf: String,
    rx: Receiver<u8>,
}

impl DexterousDevInstall {
    fn ui(&mut self, ui: &mut Ui) {
        utils::display_terminal(&mut self.buf, self.rx.clone(), ui);
    }
}

impl Default for DexterousDevInstall {
    fn default() -> Self {
        let mut command = Command::new("cargo");
        command.arg("install").arg("dexterous_developer_cli");

        let (rx, child) = utils::command_channels(command);

        DexterousDevInstall {
            child,
            buf: "".to_string(),
            rx,
        }
    }
}

pub struct ProjectRunner {
    running: ProjectItem,
    terminal_string: String,
    first_run: bool,
    rx: Option<Receiver<u8>>,
    child: Option<Child>,
}

impl ProjectRunner {
    pub fn run(&mut self, ui: &mut Ui) {
        if self.first_run {
            self.first_run = false;
            let mut command = Command::new("cargo");
            command.arg("run");
            command.arg("--");
            command.arg(self.running.dir_entry.path());
            command.current_dir(get_hotreload_dir());
            let (rx, child) = utils::command_channels(command);
            self.rx.replace(rx);
            self.child.replace(child);
        }
        utils::display_terminal(&mut self.terminal_string, self.rx.clone().unwrap(), ui);
    }
}

pub struct ProjectItem {
    name: String,
    dir_entry: DirEntry,
}
struct ProjectViewer {
    items_list: Vec<ProjectItem>,
    selected_item: Option<usize>,
    dropdown_buf_field: String,
    first_run: bool,
    show_create_project_popup: bool,
    create_project_text: String,
    selected_template: Templates,
    switch_selected: Option<usize>,
}

impl Default for ProjectViewer {
    fn default() -> Self {
        let mut project_viewer = ProjectViewer {
            items_list: vec![],
            selected_item: None,
            dropdown_buf_field: "".to_string(),
            first_run: true,
            show_create_project_popup: false,
            create_project_text: "".to_string(),
            selected_template: Templates::StandardHotReloadTemplate,
            switch_selected: None,
        };
        project_viewer.scan();
        project_viewer
    }
}

impl AsRef<str> for ProjectItem {
    fn as_ref(&self) -> &str {
        &self.name.as_str()
    }
}
impl ProjectViewer {
    pub fn add_to_fonts(ui: &mut Ui) {
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        ui.ctx().set_fonts(fonts);
    }
    pub fn scan(&mut self) {
        let path_buf = ProjectDirs::from("com", "malek", "bevy_editor").unwrap();
        let path_buf = path_buf.data_dir();
        std::fs::create_dir_all(path_buf).unwrap();
        self.items_list.clear();
        for dir in std::fs::read_dir(path_buf).unwrap() {
            if let Ok(dir) = dir {
                self.items_list.push(ProjectItem {
                    name: dir.file_name().to_str().unwrap().parse().unwrap(),
                    dir_entry: dir,
                })
            }
        }
    }
    pub fn projects(&mut self, ui: &mut Ui) {
        let new_project_popup = Modal::new(ui.ctx(), "create project modal");
        new_project_popup.show(|ui| {
            new_project_popup.title(ui, "Create Project");
            new_project_popup.frame(ui, |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.create_project_text)
                        .hint_text("new_project"),
                );
                self.create_project_text = self.create_project_text.replace(" ", "_");
            });
            new_project_popup.buttons(ui, |ui| {
                if ui.button("Close").clicked() {
                    new_project_popup.close();
                }
                if ui.button("Create").clicked() {
                    let mut path_buf = ProjectDirs::from("com", "malek", "bevy_editor")
                        .unwrap()
                        .data_dir()
                        .to_path_buf();
                    path_buf.push(&self.create_project_text);
                    let mut template = Template::get_standard_template();
                    for template in template.file_templates.iter_mut() {
                        let mut path_buf = path_buf.clone();
                        path_buf.push(template.relative_path.as_path());
                        let mut dir_path = path_buf.clone();
                        dir_path.pop();
                        std::fs::create_dir_all(dir_path.as_path()).unwrap();
                        let mut file = std::fs::OpenOptions::new()
                            .create(true)
                            .write(true)
                            .read(true)
                            .open(path_buf)
                            .unwrap();
                        file.write_all(template.contents).unwrap();
                    }
                    new_project_popup.close();
                    self.scan();
                }
            });
        });
        let remove_project_popup = Modal::new(ui.ctx(), "remove project modal");
        remove_project_popup.show(|ui| {
            remove_project_popup.title(
                ui,
                format!(
                    "Are you sure you want to remove {}",
                    self.dropdown_buf_field
                ),
            );
            remove_project_popup.buttons(ui, |ui| {
                if ui
                    .button("wtf no of course not it was an accident")
                    .clicked()
                {
                    remove_project_popup.close();
                }
                if ui.button("Yes Delete the Project").clicked() {
                    if let Some(selected) = self.selected_item {
                        if let Some(selected) = self.items_list.get(selected) {
                            std::fs::remove_dir_all(selected.dir_entry.path()).unwrap();
                            self.selected_item.take();
                            self.dropdown_buf_field = String::new();
                        }
                    }
                    self.scan();
                    remove_project_popup.close();
                }
            });
        });
        let rect = ui.label("projects").rect;
        ui.add(
            DropDownBox::from_iter(
                &self.items_list,
                "filter projects",
                &mut self.dropdown_buf_field,
                |ui, text| ui.selectable_label(false, text),
            )
            .filter_by_input(true)
            .select_on_focus(true),
        );
        let mut have_item = false;
        for (i, item) in self.items_list.iter().enumerate() {
            if item.name.as_str() == self.dropdown_buf_field.as_str() {
                self.selected_item.replace(i);
                have_item = true;
            }
        }
        if !have_item {
            self.selected_item.take();
        }
        let num = (self.items_list.len() as f32).sqrt() as i32;
        egui::Grid::new("project grid")
            .spacing(Vec2::new(15.0, 15.0))
            .show(ui, |ui| {
                let mut i = 0;
                ui.vertical_centered_justified(|ui| {
                    for (i2, item) in self.items_list.iter().enumerate() {
                        if ui
                            .add(
                                egui::Button::new(item.name.as_str())
                                    .min_size(Vec2::new(100.0, 100.0)),
                            )
                            .clicked()
                        {
                            self.dropdown_buf_field = item.name.clone();
                            self.selected_item.replace(i2);
                        }
                        if i >= num {
                            ui.end_row();
                            i = 0;
                        }
                        i += 1;
                    }
                });
            });
        egui::SidePanel::right("right panel")
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.vertical_centered_justified(|ui| {
                    if ui
                        .add_enabled(self.selected_item.is_some(), egui::Button::new("Edit"))
                        .clicked()
                    {}
                    if ui
                        .add_enabled(self.selected_item.is_some(), egui::Button::new("Run"))
                        .clicked()
                    {
                        if let Some(index) = self.selected_item {
                            self.switch_selected.replace(index);
                            return;
                        }
                    }
                    if ui
                        .add_enabled(self.selected_item.is_some(), egui::Button::new("Remove"))
                        .clicked()
                    {
                        remove_project_popup.open();
                    }

                    ui.add_space(rect.height());

                    if ui
                        .button(egui::RichText::new(format!(
                            "{} Scan",
                            egui_phosphor::regular::FOLDER
                        )))
                        .clicked()
                    {
                        self.scan();
                    }
                    if ui
                        .button(egui::RichText::new(format!(
                            "{} New Project",
                            egui_phosphor::regular::FILE_PLUS
                        )))
                        .clicked()
                    {
                        new_project_popup.open();
                    }
                });
            });
    }
    pub fn templates(&mut self, ui: &mut Ui) {
        ui.label("No other templates exist yet");
        ui.add_space(20.0);
        ui.radio_value(
            &mut self.selected_template,
            Templates::StandardHotReloadTemplate,
            "Standard HotReload Template",
        );
    }
}
#[derive(PartialEq)]
pub enum Templates {
    StandardHotReloadTemplate,
}

impl TabViewer for ProjectViewer {
    type Tab = String;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.as_str().into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        if self.first_run {
            self.first_run = false;
            Self::add_to_fonts(ui);
            egui_extras::install_image_loaders(ui.ctx());
        }
        match tab.as_str() {
            "Projects" => self.projects(ui),
            "Templates" => self.templates(ui),
            _ => panic!(),
        }
    }
}

impl MyApp {
    pub fn new() -> Self {
        let mut tree = DockState::new(vec!["Projects".to_string(), "Templates".to_string()]);
        Self {
            tree,
            app_states: AppStates::DexterousDevInstall(DexterousDevInstall::default()),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut switch_self = None;
        match &mut self.app_states {
            AppStates::ProjectViewer(project_viewer) => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    DockArea::new(&mut self.tree)
                        .style(Style::from_egui(ui.style().as_ref()))
                        .draggable_tabs(false)
                        .show_close_buttons(false)
                        .show_inside(ui, project_viewer);
                });
                if let Some(selected) = project_viewer.switch_selected.clone() {
                    let running = project_viewer.items_list.remove(selected);
                    switch_self.replace(AppStates::ProjectRunner(ProjectRunner {
                        running,
                        terminal_string: "".to_string(),
                        first_run: true,
                        rx: None,
                        child: None,
                    }));
                }
            }
            AppStates::ProjectRunner(project_runner) => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    project_runner.run(ui);
                });
                if let Some(child) = project_runner.child.as_mut() {
                    if let Ok(c) = child.try_wait() {
                        if c.is_some() {
                            switch_self.replace(AppStates::ProjectViewer(ProjectViewer::default()));
                        }
                    }
                }
            }
            AppStates::DexterousDevInstall(dexterous_dev_install) => {
                egui::CentralPanel::default().show(ctx, |ui| {
                  dexterous_dev_install.ui(ui);
                });
                if let Ok(c) = dexterous_dev_install.child.try_wait() {
                    if c.is_some() {
                        switch_self.replace(AppStates::ProjectViewer(ProjectViewer::default()));
                    }
                }
            }
        }
        if let Some(running) = switch_self {
            self.app_states = running;
        }
    }
}
