#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod catalog;
mod download;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1180.0, 780.0])
            .with_min_inner_size([920.0, 640.0])
            .with_title("Doctor AI")
            .with_icon(std::sync::Arc::new(load_icon())),
        ..Default::default()
    };
    eframe::run_native("Doctor AI", options, Box::new(|cc| {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = egui::Color32::from_rgb(12, 22, 38);
        visuals.window_fill = egui::Color32::from_rgb(15, 28, 46);
        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(25, 43, 62);
        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(34, 76, 91);
        visuals.widgets.active.bg_fill = egui::Color32::from_rgb(25, 145, 135);
        visuals.selection.bg_fill = egui::Color32::from_rgb(25, 145, 135);
        cc.egui_ctx.set_visuals(visuals);
        Ok(Box::new(app::DoctorApp::new()))
    }))
}

fn load_icon() -> egui::IconData {
    let mut rgba = Vec::with_capacity(256 * 256 * 4);
    for y in 0..256 {
        for x in 0..256 {
            let dx = x as f32 - 128.0;
            let dy = y as f32 - 128.0;
            let inside = (dx * dx + dy * dy).sqrt() < 118.0;
            let cross = (x > 110 && x < 146 && y > 54 && y < 202) || (x > 54 && x < 202 && y > 110 && y < 146);
            let c = if cross { [56, 211, 190, 255] } else if inside { [12, 39, 62, 255] } else { [0, 0, 0, 0] };
            rgba.extend_from_slice(&c);
        }
    }
    egui::IconData { rgba, width: 256, height: 256 }
}
