use super::run;
use crate::Config;

#[allow(dead_code)]
pub trait WindowSurface {
    type Renderer: femtovg::Renderer + 'static;
    fn resize(&mut self, width: u32, height: u32);
    fn present(&self, canvas: &mut femtovg::Canvas<Self::Renderer>);
}

pub mod opengl;
pub type Renderer = femtovg::renderer::OpenGl;

pub fn start(config: Config) {
    opengl::start_opengl(config)
}
