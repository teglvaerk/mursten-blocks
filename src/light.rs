use mursten::{Backend, Data, Updater};
use nalgebra::*;

pub struct Light {
    pub point: Point3<f32>,
    pub color: Vector3<f32>,
    pub strength: f32, // Cambiar esto por un coeficiente que relacione la distancia con la intensidad
}

impl Light {
    pub fn new(point: Point3<f32>, color: Vector3<f32>, strength: f32) -> Self {
        Self { point, color, strength }
    }
}

pub trait GetLights {
    fn get_light(&self) -> Light;
}

pub struct LightUpdater {}

impl LightUpdater {
    pub fn new() -> Self {
        LightUpdater {}
    }
}

impl<B, D> Updater<B, D> for LightUpdater
where
    D: Data + GetLights,
    B: Backend<D> + backend::SetLights,
{
    fn update(&mut self, backend: &mut B, data: &mut D) {
        let light = data.get_light();
        backend.set_light(light);
    }
}

pub mod backend {
    pub trait SetLights {
        fn set_light(&mut self, light: super::Light);
    }
}

