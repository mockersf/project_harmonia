use bevy::{
    prelude::*,
    render::mesh::{Indices, VertexAttributeValues},
};

#[derive(Default)]
pub(crate) struct DynamicMesh {
    pub(crate) positions: Vec<[f32; 3]>,
    pub(crate) uvs: Vec<[f32; 2]>,
    pub(crate) normals: Vec<[f32; 3]>,
    pub(crate) indices: Vec<u32>,
}

impl DynamicMesh {
    pub(crate) fn take(mesh: &mut Mesh) -> Self {
        let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.remove_attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("all wall meshes should have positions");
        };
        let Some(VertexAttributeValues::Float32x2(uvs)) =
            mesh.remove_attribute(Mesh::ATTRIBUTE_UV_0)
        else {
            panic!("all wall meshes should have UVs");
        };
        let Some(VertexAttributeValues::Float32x3(normals)) =
            mesh.remove_attribute(Mesh::ATTRIBUTE_NORMAL)
        else {
            panic!("all wall meshes should have normals");
        };
        let Some(Indices::U32(indices)) = mesh.remove_indices() else {
            panic!("all wall meshes should have U32 indices");
        };

        Self {
            positions,
            uvs,
            normals,
            indices,
        }
    }

    pub(crate) fn vertices_count(&self) -> u32 {
        self.positions
            .len()
            .try_into()
            .expect("vertices should fit u32")
    }

    pub(crate) fn clear(&mut self) {
        self.positions.clear();
        self.uvs.clear();
        self.normals.clear();
        self.indices.clear();
    }

    pub(crate) fn apply(self, mesh: &mut Mesh) {
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_indices(Indices::U32(self.indices))
    }
}