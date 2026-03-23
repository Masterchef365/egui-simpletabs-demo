use egui::{DragValue, Layout};
use egui_simpletabs::{
    buttons::{play_pause_button, reset_step_button, single_step_button},
    dial::{Dial, DialPosition, ScaleMarking, choice},
    metric::{edit_metric_f64, metric_prefix_dragvalue},
    tabs::TabWidgetExt,
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
    Everything,
    Dial,
    Metric,
    Buttons,
}

pub struct TemplateApp {
    tab: Tab,
    volts: f64,
    paused: bool,
}

impl TemplateApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            tab: Tab::Everything,
            volts: 5.0,
            paused: true,
        }
    }
}

impl eframe::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add_tab(&mut self.tab, Tab::Everything, "Everything");
                ui.add_tab(&mut self.tab, Tab::Dial, "Dial");
                ui.add_tab(&mut self.tab, Tab::Metric, "Metric");

                ui.with_layout(Layout::right_to_left(Default::default()), |ui| {
                    ui.add_tab(&mut self.tab, Tab::Buttons, "Buttons");
                    ui.cap_tabs();
                });
            });

            match self.tab {
                Tab::Everything => self.show_everything(ui),
                Tab::Dial => self.show_dials(ui),
                Tab::Metric => self.show_metric(ui),
                Tab::Buttons => self.show_buttons(ui),
            }
        });
    }
}

impl TemplateApp {
    fn show_everything(&mut self, ui: &mut egui::Ui) {
        self.show_buttons(ui);
        self.show_dials(ui);
        self.show_metric(ui);
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
            .with_position(
                DialPosition::new(10)
                    .label("Ten")
            );

        ui.horizontal(|ui| {
            ui.add(dial);

            choice(ui, &mut self.paused, &[
                (false, "Run"), (true, "Pause"),
            ]);
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
}