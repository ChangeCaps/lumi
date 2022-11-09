#![allow(dead_code)]

use lumi_material::StandardMaterial;

pub fn material_window(ctx: &egui::Context, material: &mut StandardMaterial) {
    egui::Window::new("Material").show(ctx, |ui| {
        egui::Grid::new("grid").show(ui, |ui| {
            ui.label("Thickness");
            ui.add(egui::Slider::new(&mut material.thickness, 0.0..=1.0));
            ui.end_row();

            ui.add(egui::Checkbox::new(&mut material.subsurface, "Subsurface"));
            ui.end_row();

            ui.label("Subsurface Power");
            ui.add(egui::Slider::new(
                &mut material.subsurface_power,
                0.0..=10.0,
            ));
            ui.end_row();

            ui.label("Subsurface Color");
            egui::color_picker::color_edit_button_rgb(
                ui,
                lumi_util::bytemuck::cast_mut(&mut material.subsurface_color),
            );
            ui.end_row();

            ui.label("Base Color");
            egui::color_picker::color_edit_button_rgba(
                ui,
                lumi_util::bytemuck::cast_mut(&mut material.base_color),
                egui::color_picker::Alpha::OnlyBlend,
            );
            ui.end_row();

            ui.label("Metallic");
            ui.add(egui::Slider::new(&mut material.metallic, 0.0..=1.0));
            ui.end_row();

            ui.label("Roughness");
            ui.add(egui::Slider::new(&mut material.roughness, 0.0..=1.0));
            ui.end_row();

            ui.label("Clearcoat");
            ui.add(egui::Slider::new(&mut material.clearcoat, 0.0..=1.0));
            ui.end_row();

            ui.label("Clearcoat Roughness");
            ui.add(egui::Slider::new(
                &mut material.clearcoat_roughness,
                0.0..=1.0,
            ));
            ui.end_row();

            ui.label("Emissive");
            egui::color_picker::color_edit_button_rgb(
                ui,
                lumi_util::bytemuck::cast_mut(&mut material.emissive),
            );
            ui.end_row();

            ui.label("Transmission");
            ui.add(egui::Slider::new(&mut material.transmission, 0.0..=1.0));
            ui.end_row();

            ui.label("Ior");
            ui.add(egui::Slider::new(&mut material.ior, 1.0..=4.0));
            ui.end_row();

            ui.label("Absorption");
            egui::color_picker::color_edit_button_rgb(
                ui,
                lumi_util::bytemuck::cast_mut(&mut material.absorption),
            );
            ui.end_row();
        });
    });
}
