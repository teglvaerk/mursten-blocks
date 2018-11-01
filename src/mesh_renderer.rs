use mursten::{Backend, Data, Renderer};
use nalgebra::*;
use std::vec;

use super::geometry::Mesh;


pub struct MeshRenderer {}

impl MeshRenderer {
    pub fn new() -> Self {
        MeshRenderer {}
    }
}

impl<B, D> Renderer<B, D> for MeshRenderer
where
    D: Data + GetMeshes,
    B: Backend<D> + backend::RenderMesh,
{
    fn render(&mut self, backend: &mut B, data: &D) {
        for mesh in data.mesh_iter() {
            backend.queue_render(mesh.transform(), mesh.mesh());
        }
    }
}

pub trait GetMeshes {
    fn mesh_iter<'a>(&'a self) -> vec::IntoIter<&IntoMesh>;
}

// TODO: Find a better name for this
pub trait IntoMesh {
    fn transform(&self) -> Matrix4<f32> {
        Matrix4::identity()
    }
    fn mesh(&self) -> Mesh;
}

pub mod backend {
    use nalgebra::*;

    pub trait RenderMesh {
        fn queue_render(&mut self, Matrix4<f32>, super::Mesh);
    }
}

