use base58_bytes::BaseBytesConverter;
use date_timestamp::DateConverter;

pub mod base58_bytes;
pub mod date_timestamp;

#[derive(Default)]
pub struct ConverterApp {
    base_bytes_converter: BaseBytesConverter,
    date_timestamp_converter: DateConverter,
}

impl eframe::App for ConverterApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("Base Bytes Converter").show(ctx, |ui| {
            self.base_bytes_converter.ui(ui);
        });

        egui::TopBottomPanel::top("Datetime Converter").show(ctx, |ui| {
            self.date_timestamp_converter.ui(ui);
        });

        egui::CentralPanel::default()
            .frame(egui::Frame::dark_canvas(&ctx.style()))
            .show(ctx, |_| {
                // TODO: Other panels
            });
    }
}
