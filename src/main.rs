use egui::{Color32, ComboBox, Context, DragValue, Layout, Stroke, Ui, global_theme_preference_buttons};
use egui_simpletabs::{
    buttons::{play_pause_button, reset_step_button, single_step_button}, dial::{Dial, DialPosition, DragMode, ScaleMarking, choice}, groupbox::{FrameGroupBoxExt, GroupBox, UiGroupBoxExt}, metric::{edit_metric_f64, metric_prefix_dragvalue}, tabs::TabWidgetExt, utils::IndecisiveOption
};

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0])
            .with_icon(
                // NOTE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Ok(Box::new(TemplateApp::new(cc)))),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(TemplateApp::new(cc)))),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}

#[derive(PartialEq, Eq)]
enum Tab {
    Home,
    Dial,
    DialEditor,
    Metric,
    Buttons,
    Groups,
}

pub struct TemplateApp {
    tab: Tab,
    volts: f64,
    paused: bool,
    value: f64,
    example_text: String,

    drag_mode: DragMode,

    min: IndecisiveOption<f32>,
    max: IndecisiveOption<f32>,

    invert: bool,

    underline: bool,

    origin_angle: f64,
    origin_value: f64,

    mouse_sensitivity: f64,

    value_per_radian: f64,

    show_livezone: bool,

    snap: IndecisiveOption<f32>,

    value_int: i32,

    value_positional: f32,
}

impl TemplateApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            tab: Tab::Home,
            volts: 5.0,
            paused: true,
            value: 1f64,
            example_text: "Example text".into(),

            drag_mode: DragMode::default(),

            min: Some(-2.0).into(),
            max: Some(2.0).into(),

            invert: false,

            underline: true,

            origin_angle: -std::f64::consts::FRAC_PI_2,
            origin_value: 0.0,

            mouse_sensitivity: 5e-2,

            value_per_radian: 1.0,

            show_livezone: true,

            snap: Some(0.05).into(),

            value_int: 1,

            value_positional: 1.5,
        }
    }
}

impl eframe::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.tab == Tab::DialEditor {
            self.dial_editor_cfg(ctx);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add_tab(&mut self.tab, Tab::Home, "Everything");
                ui.add_tab(&mut self.tab, Tab::Dial, "Dial");
                ui.add_tab(&mut self.tab, Tab::DialEditor, "Dial Editor");
                ui.add_tab(&mut self.tab, Tab::Metric, "Metric");
                ui.add_tab(&mut self.tab, Tab::Groups, "Groups");

                ui.with_layout(Layout::right_to_left(Default::default()), |ui| {
                    ui.add_tab(&mut self.tab, Tab::Buttons, "Buttons");
                    ui.cap_tabs();
                });
            });

            match self.tab {
                Tab::Home => self.show_everything(ui),
                Tab::Dial => self.show_dials(ui),
                Tab::Metric => self.show_metric(ui),
                Tab::Buttons => self.show_buttons(ui),
                Tab::DialEditor => self.dial_editor_view(ui),
                Tab::Groups => self.show_groups(ui),
            }
        });
    }
}

impl TemplateApp {
    fn show_everything(&mut self, ui: &mut egui::Ui) {
        self.show_buttons(ui);
        self.show_dials(ui);
        self.show_metric(ui);
        self.show_groups(ui);
    }

    fn show_dials(&mut self, ui: &mut egui::Ui) {
        let dial = Dial::new(&mut self.volts)
            .value_per_radian(12.0 / std::f32::consts::TAU as f64)
            .max_value(Some(10.0))
            .min_value(Some(-1.0))
            .origin_angle(1.0)
            .with_scale_marking(ScaleMarking::default())
            .with_position(
                DialPosition::new(0)
                    .label("Zero")
                    .color(egui::Color32::DARK_GREEN),
            )
            .with_position(
                DialPosition::new(5)
                    .label("Five")
                    .color(egui::Color32::DARK_RED),
            )
            .with_position(DialPosition::new(10).label("Ten"));

        ui.horizontal(|ui| {
            ui.add(dial);

            ui.group_box("Run state", |ui| {
                choice(ui, &mut self.paused, &[(false, "Run"), (true, "Pause")]);
            });
        });

        ui.label("Double click labels to jump to their value.");
    }

    fn show_metric(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Voltage: ");
            ui.add(edit_metric_f64(&mut self.volts, "V"));
        });
    }

    fn show_buttons(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            play_pause_button(ui, &mut self.paused);
            single_step_button(ui);

            if reset_step_button(ui).clicked() {
                self.volts = 0.0;
            }
        });
    }

    fn dial_editor_view(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.heading("Dial (float)");

            let mut dial = Dial::new(&mut self.value)
                .drag_mode(self.drag_mode)
                .value_per_radian(self.value_per_radian)
                .min_value(self.min.into_option())
                .max_value(self.max.into_option())
                .invert(self.invert)
                .origin_angle(self.origin_angle)
                .origin_value(self.origin_value)
                .mouse_sensitivity(self.mouse_sensitivity)
                .show_livezone(self.show_livezone)
                .with_scale_marking(ScaleMarking::default().with_interval(0.5))
                .with_position(
                    DialPosition::new(0)
                        .label("Zero")
                        .snap(self.snap.into())
                        .underline(self.underline)
                        .color(Color32::DARK_GREEN),
                )
                .with_position(
                    DialPosition::new(1)
                        .label("One")
                        .snap(self.snap.into())
                        .underline(self.underline),
                );

            if let Some(min) = self.min.into_option() {
                dial = dial.with_position(
                    DialPosition::new(min)
                        .label("Min")
                        .snap(self.snap.into())
                        .underline(self.underline),
                );
            }

            if let Some(max) = self.max.into_option() {
                dial = dial.with_position(
                    DialPosition::new(max)
                        .label("Max")
                        .snap(self.snap.into())
                        .underline(self.underline),
                );
            }

            ui.add(dial);
            ui.add(DragValue::new(&mut self.value).speed(1e-2));
        });

        ui.group(|ui| {
            ui.heading("Dial (integer value)");
            let mut dial = Dial::new(&mut self.value_int)
                .drag_mode(self.drag_mode)
                .value_per_radian(self.value_per_radian)
                .min_value(self.min.into_option().map(|v| v.floor()))
                .max_value(self.max.into_option().map(|v| v.ceil()))
                .invert(self.invert)
                .origin_angle(self.origin_angle)
                .origin_value(self.origin_value)
                .mouse_sensitivity(self.mouse_sensitivity * 20.0)
                .show_livezone(self.show_livezone)
                .with_scale_marking(ScaleMarking::default().with_interval(1.0))
                .knob_style(egui_simpletabs::dial::KnobStyle::Circular)
                .with_position(
                    DialPosition::new(0)
                        .label("Zero")
                        .snap(self.snap.into())
                        .underline(self.underline)
                        .color(Color32::DARK_GREEN),
                )
                .with_position(
                    DialPosition::new(1)
                        .label("One")
                        .snap(self.snap.into())
                        .underline(self.underline),
                );

            if let Some(min) = self.min.into_option() {
                dial = dial.with_position(
                    DialPosition::new(min.floor())
                        .label("Min")
                        .snap(self.snap.into())
                        .underline(self.underline),
                );
            }

            if let Some(max) = self.max.into_option() {
                dial = dial.with_position(
                    DialPosition::new(max.ceil())
                        .label("Max")
                        .snap(self.snap.into())
                        .underline(self.underline),
                );
            }

            ui.add(dial);
            ui.add(DragValue::new(&mut self.value_int).speed(1e-2));
        });

        ui.group(|ui| {
            ui.heading("Dial (positional values)");
            let mut dial = Dial::new(&mut self.value_positional)
                .drag_mode(self.drag_mode)
                .value_per_radian(self.value_per_radian)
                .min_value(self.min.into_option())
                .max_value(self.max.into_option())
                .invert(self.invert)
                .origin_angle(self.origin_angle)
                .origin_value(self.origin_value)
                .mouse_sensitivity(self.mouse_sensitivity * 20.0)
                .show_livezone(self.show_livezone)
                .turning_mode(egui_simpletabs::dial::TurningMode::Positional)
                .knob_style(egui_simpletabs::dial::KnobStyle::Fluted {
                    n_segments: 18,
                    depth: 0.1,
                })
                .with_position(
                    DialPosition::new(0)
                        .label("Zero")
                        .snap(self.snap.into())
                        .underline(self.underline)
                        .color(Color32::DARK_GREEN),
                )
                .with_position(
                    DialPosition::new(1.5)
                        .label("1.5")
                        .snap(self.snap.into())
                        .underline(self.underline),
                );

            if let Some(min) = self.min.into_option() {
                dial = dial.with_position(
                    DialPosition::new(min)
                        .label("Min")
                        .snap(self.snap.into())
                        .underline(self.underline),
                );
            }

            if let Some(max) = self.max.into_option() {
                dial = dial.with_position(
                    DialPosition::new(max)
                        .label("Max")
                        .snap(self.snap.into())
                        .underline(self.underline),
                );
            }

            ui.add(dial);
            ui.add(DragValue::new(&mut self.value_positional).speed(1e-2));
        });

        ui.label("Double click labels to snap to their position");
    }

    fn dial_editor_cfg(&mut self, ctx: &Context) {
        egui::SidePanel::left("cfg").show(ctx, |ui| {
            global_theme_preference_buttons(ui);

            ui.group(|ui| {
                ui.strong("Scale and range");
                ui.horizontal(|ui| {
                    ui.label("Min value");
                    self.min
                        .show(ui, |ui, min| ui.add(DragValue::new(min).speed(1e-2)));
                });

                ui.horizontal(|ui| {
                    ui.label("Max value");
                    self.max
                        .show(ui, |ui, max| ui.add(DragValue::new(max).speed(1e-2)));
                });

                ui.horizontal(|ui| {
                    ui.label("Scale");
                    ui.add(DragValue::new(&mut self.value_per_radian).speed(1e-2));
                });

                ui.checkbox(&mut self.invert, "Invert");

                ui.horizontal(|ui| {
                    ui.label("Origin angle: ");
                    ui.add(DragValue::new(&mut self.origin_angle).speed(1e-2));
                });

                ui.horizontal(|ui| {
                    ui.label("Origin value: ");
                    ui.add(DragValue::new(&mut self.origin_value).speed(1e-2));
                });
            });

            ui.group(|ui| {
                ui.strong("Drawing");
                ui.checkbox(&mut self.underline, "Underline");
                ui.checkbox(&mut self.show_livezone, "Show live zone");
            });

            ui.group(|ui| {
                ui.strong("Interactivity");
                ui.horizontal(|ui| {
                    ui.label("Snap: ");
                    self.snap.show(ui, |ui, snap_thresh| {
                        ui.add(
                            DragValue::new(snap_thresh)
                                .prefix("Tolerance: ")
                                .speed(1e-2),
                        )
                    });
                    //ui.checkbox(&mut has_snap, "Snap");
                    //ui.add_enabled(has_snap, DragValue::new(&mut snap_thresh).speed(1e-2));
                });

                ui.horizontal(|ui| {
                    ui.label("Mouse sensitivity");
                    ui.add(DragValue::new(&mut self.mouse_sensitivity).speed(1e-2));
                });

                ComboBox::new("drag", "Drag mode")
                    .selected_text(format!("{:?}", self.drag_mode))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.drag_mode,
                            DragMode::CoordinateY,
                            "Coordinate Y",
                        );
                        ui.selectable_value(
                            &mut self.drag_mode,
                            DragMode::CoordinateX,
                            "Coordinate X",
                        );
                        ui.selectable_value(&mut self.drag_mode, DragMode::Radial, "Radial");
                        ui.selectable_value(
                            &mut self.drag_mode,
                            DragMode::DistanceFromCenter,
                            "Distance From Center",
                        );
                    })
            });
        });
    }

    fn show_groups(&mut self, ui: &mut egui::Ui) {
        ui.text_edit_singleline(&mut self.example_text);
        global_theme_preference_buttons(ui);

        ui.group_box("This is a group box", |ui| {
            ui.label("And it has stuff in it");
        });

        egui::Frame::group(ui.style())
            .fill(Color32::BLUE)
            .corner_radius(30.0)
            .outer_margin(30.0)
            .inner_margin(30.0)
            .group_box(&self.example_text)
            .text_color(Color32::RED)
            .stroke(Stroke::new(0.5, Color32::RED))
            .show(ui, |ui| {
                ui.label("This statement is false or whatever");
            });
    }
}
