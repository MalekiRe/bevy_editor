use bevy::app::AppExit;
use bevy::input::keyboard::Key;
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::prelude::*;
use bevy::window::{WindowRef, WindowResolution};
use bevy::DefaultPlugins;
use bevy_editor_pls::default_windows::add::AddWindow;
use bevy_editor_pls::default_windows::assets::AssetsWindow;
use bevy_editor_pls::default_windows::cameras::CameraWindow;
use bevy_editor_pls::default_windows::debug_settings::DebugSettingsWindow;
use bevy_editor_pls::default_windows::diagnostics::DiagnosticsWindow;
use bevy_editor_pls::default_windows::gizmos::GizmoWindow;
use bevy_editor_pls::default_windows::hierarchy::HierarchyWindow;
use bevy_editor_pls::default_windows::inspector::InspectorWindow;
use bevy_editor_pls::default_windows::renderer::RendererWindow;
use bevy_editor_pls::default_windows::resources::ResourcesWindow;
use bevy_editor_pls::default_windows::scenes::SceneWindow;
use bevy_editor_pls::egui::{Color32, RichText, ScrollArea, Ui};
use bevy_editor_pls::{controls, egui, egui_dock, EditorWindowPlacement};
use bevy_editor_pls_core::editor_window::{EditorWindow, EditorWindowContext};
use bevy_editor_pls_core::{editor, AddEditorWindow};
use cansi::v3::categorise_text;
use dexterous_developer::{
    dexterous_developer_setup, hot_bevy_main, InitialPlugins, ReloadMode, ReloadSettings,
    ReloadableApp, ReloadableAppContents, ReloadableElementsSetup,
};
use egui_code_editor::{ColorTheme, Syntax};
use std::fs::DirEntry;
use std::io::{Bytes, Read};
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::thread;

#[hot_bevy_main]
fn bevy_main(initial_plugins: impl InitialPlugins) {
    if std::env::var("ONLY_UI").is_ok() {
        App::new()
            .add_plugins(initial_plugins.initialize::<DefaultPlugins>())
            .add_plugins(EditorPlugin::default())
            .add_editor_window::<CodeEditor>()
            .add_systems(Startup, |mut commands: Commands| {
                commands.spawn(Camera3dBundle::default());
            })
            .add_systems(Last, |mut app_exit: EventReader<AppExit>| {
                if unsafe { MY_THING } == 0 {
                    if !app_exit.is_empty() {
                        exit(0);
                    }
                }
            })
            .run();
    } else {
        App::new()
            .add_plugins(initial_plugins.initialize::<DefaultPlugins>())
            .add_plugins(EditorPlugin::default())
            .add_editor_window::<CodeEditor>()
            .setup_reloadable_elements::<reloadable>()
            .add_systems(Last, |mut app_exit: EventReader<AppExit>| {
                if unsafe { MY_THING } == 0 {
                    if !app_exit.is_empty() {
                        exit(0);
                    }
                }
            })
            .run();
    }
}

static mut MY_THING: i32 = 0;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(Color::rgb_u8(21, 14, 25)),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        GetRidOf,
    ));
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        GetRidOf,
    ));
}

#[dexterous_developer_setup]
fn reloadable(app: &mut ReloadableAppContents) {
    app.reset_setup::<GetRidOf, _>(setup);
}

#[derive(Component)]
struct GetRidOf;

pub struct CodeEditor;
pub struct CodeEditorState {
    code: String,
    selected_file: Option<PathBuf>,
}

fn display_dir(ui: &mut Ui, dir: DirEntry) -> Option<PathBuf> {
    if dir.file_type().unwrap().is_dir() {
        let mut path_buf = None;
        ui.collapsing(dir.file_name().to_str().unwrap(), |ui| {
            for dir in std::fs::read_dir(dir.path()).unwrap().into_iter() {
                if let Ok(dir) = dir {
                    if let Some(buf) = display_dir(ui, dir) {
                        path_buf.replace(buf);
                    }
                }
            }
        });
        path_buf
    } else {
        if ui.button(dir.file_name().to_str().unwrap()).clicked() {
            Some(dir.path())
        } else {
            None
        }
    }
}

impl EditorWindow for CodeEditor {
    type State = CodeEditorState;
    const NAME: &'static str = "Code editor";

    fn ui(world: &mut World, mut cx: EditorWindowContext, ui: &mut Ui) {
        let code = cx.state_mut::<CodeEditor>().unwrap();
        ui.horizontal(|ui| {
            let mut path_buf = None;
            ui.vertical(|ui| {
                let dir = std::fs::read_dir(std::env::current_dir().unwrap()).unwrap();
                for dir in dir.into_iter() {
                    if let Ok(dir) = dir {
                        if let Some(buf) = display_dir(ui, dir) {
                            path_buf.replace(buf);
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
                        egui::Event::Key {
                            key,
                            physical_key,
                            pressed,
                            repeat,
                            modifiers,
                        } => {
                            if modifiers.ctrl && key.eq(&egui::Key::S) {
                                if let Some(old) = code.selected_file.clone() {
                                    std::fs::write(old, &code.code).unwrap();
                                }
                            }
                            if modifiers.ctrl {
                                control_key = true;
                            }
                        }
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
                    }
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

impl Default for CodeEditorState {
    fn default() -> Self {
        Self {
            code: "awa".to_string(),
            selected_file: None,
        }
    }
}

pub struct Terminal;
pub struct TerminalState {
    string: Arc<Mutex<String>>,
    rich_texts: Arc<Mutex<Vec<RichText>>>,
}

impl Default for TerminalState {
    fn default() -> Self {
        let s = Arc::new(Mutex::new("".to_string()));
        let rich_texts = Arc::new(Mutex::new(Vec::new()));
        let rich_texts2 = rich_texts.clone();
        let s2 = s.clone();

        thread::spawn(move || {
            let mut stream = TcpStream::connect("127.0.0.0:8888").unwrap();
            for b in stream.bytes() {
                if let Ok(b) = b {
                    s2.lock().unwrap().push(char::from(b));
                }
                let string = s2.lock().unwrap();
                let text = categorise_text(string.as_str());
                rich_texts2.lock().unwrap().clear();
                for t in text.into_iter() {
                    let mut rich_text = egui::RichText::new(t.text);
                    if let Some(fg) = t.fg {
                        let fg = from_cansi_to_egui_color(fg);
                        rich_text = rich_text.color(fg);
                    }
                    if let Some(italics) = t.italic {
                        if italics {
                            rich_text = rich_text.italics();
                        }
                    }
                    if let Some(bg) = t.bg {
                        let bg = from_cansi_to_egui_color(bg);
                        rich_text = rich_text.background_color(bg);
                    }
                    rich_texts2.lock().unwrap().push(rich_text);
                }
            }
        });

        Self {
            string: s,
            rich_texts,
        }
    }
}

fn from_cansi_to_egui_color(color: cansi::Color) -> egui::Color32 {
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

impl EditorWindow for Terminal {
    type State = TerminalState;
    const NAME: &'static str = "Terminal";

    fn ui(world: &mut World, mut cx: EditorWindowContext, ui: &mut Ui) {
        let terminal_state = cx.state_mut::<Terminal>().unwrap();

        let scroll_to_bottom = ui.button("scroll to bottom").clicked();

        if ui.button("restart").clicked() {
            unsafe {
                MY_THING = 1;
            }
            eprint!("{}", char::from(u8::MAX));
            eprintln!("{}", char::from(u8::MAX));
            world.send_event(AppExit);
        }

        let rich_texts = terminal_state.rich_texts.lock().unwrap().clone();

        ScrollArea::new(true).show(ui, |ui| {
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
            if scroll_to_bottom {
                let label = ui.label("");
                ui.scroll_to_rect(label.rect, None);
            }
        });
    }
}

#[derive(Default)]
pub struct EditorPlugin {
    pub window: EditorWindowPlacement,
}

impl EditorPlugin {
    pub fn new() -> Self {
        EditorPlugin::default()
    }

    /// Start the editor in a new window. Use [`Window::default`] for creating a new window with default settings.
    pub fn in_new_window(mut self, window: Window) -> Self {
        self.window = EditorWindowPlacement::New(window);
        self
    }
    /// Start the editor on the second window ([`MonitorSelection::Index(1)`].
    pub fn on_second_monitor_fullscreen(self) -> Self {
        self.in_new_window(Window {
            // TODO: just use `mode: BorderlessFullscreen` https://github.com/bevyengine/bevy/pull/8178
            resolution: WindowResolution::new(1920.0, 1080.0),
            position: WindowPosition::Centered(MonitorSelection::Index(1)),
            decorations: false,
            ..Default::default()
        })
    }
}

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let window = match self.window {
            EditorWindowPlacement::New(ref window) => {
                let mut window = window.clone();
                if window.title == "Bevy App" {
                    window.title = "bevy_editor_pls".into();
                }
                let entity = app.world.spawn(window);
                WindowRef::Entity(entity.id())
            }
            EditorWindowPlacement::Window(entity) => WindowRef::Entity(entity),
            EditorWindowPlacement::Primary => WindowRef::Primary,
        };

        app.add_plugins(bevy_editor_pls_core::EditorPlugin { window });

        // if !app.is_plugin_added::<bevy_framepace::FramepacePlugin>() {
        //     app.add_plugins(bevy_framepace::FramepacePlugin);
        //     app.add_plugins(bevy_framepace::debug::DiagnosticsPlugin);
        // }

        {
            use bevy_editor_pls_default_windows::add::AddWindow;
            use bevy_editor_pls_default_windows::assets::AssetsWindow;
            use bevy_editor_pls_default_windows::cameras::CameraWindow;
            use bevy_editor_pls_default_windows::debug_settings::DebugSettingsWindow;
            use bevy_editor_pls_default_windows::diagnostics::DiagnosticsWindow;
            use bevy_editor_pls_default_windows::gizmos::GizmoWindow;
            use bevy_editor_pls_default_windows::hierarchy::HierarchyWindow;
            use bevy_editor_pls_default_windows::inspector::InspectorWindow;
            use bevy_editor_pls_default_windows::renderer::RendererWindow;
            use bevy_editor_pls_default_windows::resources::ResourcesWindow;
            use bevy_editor_pls_default_windows::scenes::SceneWindow;

            app.add_editor_window::<HierarchyWindow>();
            app.add_editor_window::<AssetsWindow>();
            app.add_editor_window::<InspectorWindow>();
            app.add_editor_window::<DebugSettingsWindow>();
            app.add_editor_window::<AddWindow>();
            app.add_editor_window::<DiagnosticsWindow>();
            app.add_editor_window::<RendererWindow>();
            app.add_editor_window::<CameraWindow>();
            app.add_editor_window::<ResourcesWindow>();
            app.add_editor_window::<SceneWindow>();
            app.add_editor_window::<GizmoWindow>();
            app.add_editor_window::<controls::ControlsWindow>();

            app.add_editor_window::<Terminal>();

            app.add_plugins(bevy::pbr::wireframe::WireframePlugin);

            app.insert_resource(controls::EditorControls::default_bindings())
                .add_systems(Update, controls::editor_controls_system);

            let mut internal_state = app.world.resource_mut::<editor::EditorInternalState>();

            let [game, _inspector] =
                internal_state.split_right::<InspectorWindow>(egui_dock::NodeIndex::root(), 0.75);
            internal_state.push_to_focused_leaf::<CodeEditor>();
            let [game, _hierarchy] = internal_state.split_left::<HierarchyWindow>(game, 0.2);
            let [_game, _bottom] = internal_state.split_many(
                game,
                0.8,
                egui_dock::Split::Below,
                &[
                    std::any::TypeId::of::<Terminal>(),
                    std::any::TypeId::of::<ResourcesWindow>(),
                    std::any::TypeId::of::<AssetsWindow>(),
                    std::any::TypeId::of::<DebugSettingsWindow>(),
                    std::any::TypeId::of::<DiagnosticsWindow>(),
                ],
            );
        }
    }
}
