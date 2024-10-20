use crate::converter::ConverterApp;

#[cfg(all(feature = "web_app", feature = "offline"))]
compile_error!("feature \"web_app\" and feature \"offline\" cannot be enabled at the same time");

#[cfg(all(not(feature = "web_app"), not(feature = "offline")))]
compile_error!("either feature \"web_app\" or feature \"offline\" must be enabled");

#[derive(Default)]
pub struct State {
    data_converter: ConverterApp,

    selected_anchor: Anchor,
}

pub struct MainApp {
    state: State,
}

impl MainApp {
    pub fn new(_: &eframe::CreationContext<'_>) -> Self {
        #[allow(unused_mut)]
        let mut slf = Self {
            state: State::default(),
        };
        slf
    }

    fn apps_iter_mut(&mut self) -> impl Iterator<Item = (&str, Anchor, &mut dyn eframe::App)> {
        let vec = vec![(
            "Base Bytes Converter",
            Anchor::BaseBytesConverter,
            &mut self.state.data_converter as &mut dyn eframe::App,
        )];
        vec.into_iter()
    }

    fn show_selected_app(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let selected_anchor = self.state.selected_anchor;
        for (_name, anchor, app) in self.apps_iter_mut() {
            if anchor == selected_anchor || ctx.memory(|mem| mem.everything_is_visible()) {
                app.update(ctx, frame);
            }
        }
    }
}

impl eframe::App for MainApp {
    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        // Give the area behind the floating windows a different color, because it looks better:
        let color = egui::lerp(
            egui::Rgba::from(visuals.panel_fill)..=egui::Rgba::from(visuals.extreme_bg_color),
            0.5,
        );
        let color = egui::Color32::from(color);
        color.to_normalized_gamma_f32()
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.show_selected_app(ctx, frame);
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
enum Anchor {
    BaseBytesConverter,
}

impl std::fmt::Display for Anchor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl From<Anchor> for egui::WidgetText {
    fn from(value: Anchor) -> Self {
        Self::RichText(egui::RichText::new(value.to_string()))
    }
}

impl Default for Anchor {
    fn default() -> Self {
        Self::BaseBytesConverter
    }
}
