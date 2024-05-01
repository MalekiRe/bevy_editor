use crate::terminal;
use crate::terminal::display_terminal;
use bevy::app::{Plugin, Update};
use bevy::prelude::{MonitorSelection, Window, WindowPosition, World};
use bevy::window::{WindowRef, WindowResolution};
use bevy_editor_pls::egui::Ui;
use bevy_editor_pls::{controls, egui_dock, EditorWindowPlacement};
use bevy_editor_pls_core::editor_window::{EditorWindow, EditorWindowContext};
use bevy_editor_pls_core::{editor, AddEditorWindow};
use crossbeam_channel::{Receiver, Sender};

pub struct Terminal;

pub struct TerminalState {
    terminal_buf: String,
    terminal_output: Receiver<u8>,
    quit_reason: Sender<u8>,
    auto_scroll: bool,
}

impl Default for TerminalState {
    fn default() -> Self {
        let (terminal_output, quit_reason) = terminal::setup_streams();
        Self {
            terminal_buf: "".to_string(),
            terminal_output,
            quit_reason,
            auto_scroll: false,
        }
    }
}

impl EditorWindow for Terminal {
    type State = TerminalState;
    const NAME: &'static str = "";

    fn ui(world: &mut World, mut cx: EditorWindowContext, ui: &mut Ui) {
        let terminal_state = cx.state_mut::<Terminal>().unwrap();

        let mut scroll_to_bottom = !terminal_state.terminal_output.is_empty();
        if terminal_state.auto_scroll {
            scroll_to_bottom = false;
        }
        ui.checkbox(&mut terminal_state.auto_scroll, "auto scroll terminal");
        display_terminal(
            &mut terminal_state.terminal_buf,
            terminal_state.terminal_output.clone(),
            ui,
            terminal_state.auto_scroll,
        );
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
            /*internal_state.push_to_focused_leaf::<CodeEditor>();*/
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
