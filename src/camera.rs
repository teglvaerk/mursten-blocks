use mursten::{Backend, Data, Updater};
use nalgebra::*;

pub struct Camera {
    pub projection: Matrix4<f32>,
}

pub trait GetCamera {
    fn get_camera<'a>(&'a self) -> (Matrix4<f32>, &'a Camera);
}

pub struct CameraUpdater {}

impl CameraUpdater {
    pub fn new() -> Self {
        CameraUpdater {}
    }
}

impl<B, D> Updater<B, D> for CameraUpdater
where
    D: Data + GetCamera,
    B: Backend<D> + backend::SetCamera,
{
    fn update(&mut self, backend: &mut B, data: &mut D) {
        let (transform, camera) = data.get_camera();
        backend.set_camera(transform, camera);
    }
}

impl Camera {
    pub fn orthographic() -> Self {
        Camera {
            projection: Orthographic3::new(-1.0, 1.0, -1.0, 1.0, 10.0, 900.0).to_homogeneous(),
        }
    }
    pub fn perspective() -> Self {
        Camera {
            projection: Perspective3::new(1.0, 1.17, 0.1, 900.0).to_homogeneous(),
        }
    }
}

pub mod backend {
    use camera::Camera;
    use nalgebra::*;

    pub trait SetCamera {
        fn set_camera(&mut self, Matrix4<f32>, &Camera);
    }
}
