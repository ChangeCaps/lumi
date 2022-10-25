use egui::Vec2;
use lumi::prelude::{Mat4, Vec3};
use winit::event::{DeviceEvent, ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent};

pub struct CameraController {
    pub rotate: bool,
    pub translate: bool,
    pub translation: Vec3,
    pub distance: f32,
    pub rotation: Vec2,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            rotate: false,
            translate: false,
            translation: Vec3::ZERO,
            distance: 2.0,
            rotation: Vec2::ZERO,
        }
    }
}

impl CameraController {
    pub fn event(&mut self, event: &Event<()>) {
        match event {
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta } => {
                    if self.rotate {
                        self.rotation -= Vec2::new(delta.0 as f32, delta.1 as f32) * 0.001;
                    }

                    if self.translate {
                        let right = self.rotation().transform_vector3(Vec3::X);
                        let down = self.rotation().transform_vector3(-Vec3::Y);

                        let delta = right * delta.0 as f32 + down * delta.1 as f32;

                        self.translation -= delta * 0.001 * self.distance;
                    }
                }
                DeviceEvent::MouseWheel { delta } => {
                    let delta = match delta {
                        MouseScrollDelta::LineDelta(_, y) => *y,
                        MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                    };

                    self.distance *= 1.0 + delta * 0.001;
                }
                _ => {}
            },
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::MouseInput { state, button, .. } => {
                    if *button == MouseButton::Right {
                        self.rotate = *state == ElementState::Pressed;
                    }

                    if *button == MouseButton::Middle {
                        self.translate = *state == ElementState::Pressed;
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    pub fn rotation(&self) -> Mat4 {
        Mat4::from_rotation_y(self.rotation.x) * Mat4::from_rotation_x(self.rotation.y)
    }

    pub fn view(&self) -> Mat4 {
        Mat4::from_translation(self.translation)
            * self.rotation()
            * Mat4::from_translation(Vec3::new(0.0, 0.0, self.distance))
    }
}
