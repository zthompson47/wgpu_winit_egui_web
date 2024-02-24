fn main() {
    pollster::block_on(wgpu_winit_egui_web::run());
}
