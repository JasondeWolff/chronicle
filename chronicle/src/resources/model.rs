use cgmath::{Vector4, Vector3, Vector2};

use crate::resources::Texture;
use crate::resources::Resource;

#[derive(Clone)]
pub struct Material {
    pub name: String,
    pub index: Option<usize>,

    pub base_color_factor: Vector4<f32>,
    pub base_color_texture: Resource<Texture>,

    pub normal_scale: f32,
    pub normal_texture: Resource<Texture>,

    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub metallic_roughness_texture: Resource<Texture>,

    pub occlusion_strength: f32,
    pub occlusion_texture: Resource<Texture>,

    pub emissive_factor: Vector3<f32>,
    pub emissive_texture: Resource<Texture>,
}

impl Default for Material {
    fn default() -> Self {
        Material {
            name: String::from("default"),
            index: None,
            base_color_factor: Vector4::<f32>::new(1.0, 1.0, 1.0, 1.0),
            base_color_texture: Resource::empty(),
            normal_scale: 1.0,
            normal_texture: Resource::empty(),
            metallic_factor: 0.0,
            roughness_factor: 1.0,
            metallic_roughness_texture: Resource::empty(),
            occlusion_strength: 1.0,
            occlusion_texture: Resource::empty(),
            emissive_factor: Vector3::<f32>::new(0.0, 0.0, 0.0),
            emissive_texture: Resource::empty(),
        }
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct Vertex {
    pub position: Vector3::<f32>,
    pub normal: Vector3::<f32>,
    pub tangent: Vector4::<f32>,
    pub tex_coord: Vector2::<f32>,
    pub tex_coord_1: Vector2::<f32>,
    pub color: Vector4::<f32>
}

impl Default for Vertex {
    fn default() -> Self {
        Vertex {
            position: Vector3::new(0.0, 0.0, 0.0),
            normal: Vector3::new(0.0, 0.0, 0.0),
            tangent: Vector4::new(0.0, 0.0, 0.0, 0.0),
            tex_coord: Vector2::new(0.0, 0.0),
            tex_coord_1: Vector2::new(0.0, 0.0),
            color: Vector4::new(0.0, 0.0, 0.0, 0.0)
        }
    }
}

#[derive(Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,

    pub min: Vector3::<f32>,
    pub max: Vector3::<f32>,

    pub material_idx: usize
}

#[derive(Clone)]
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Resource<Material>>
}