use cgmath::{Vector3, Quaternion, Matrix4, SquareMatrix, perspective, Deg};
use super::Transform;

use crate::app;

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    transform: Transform,

    aspect_ratio: Option<f32>,
    fov: f32,
    near: f32,
    far: f32,

    proj_dirty: bool,
    proj_matrix: Matrix4<f32>
}

impl Camera {
    pub fn new() -> Self {
        Camera {
             transform: Transform::new(),
             aspect_ratio: None,
             fov: 60.0,
             near: 0.1,
             far: 100.0,
             proj_dirty: true,
             proj_matrix: SquareMatrix::identity()
        }
    }

    pub fn get_translation(&self) -> &Vector3<f32> {
        &self.transform.get_translation()
    }

    pub fn get_rotation(&self) -> &Quaternion<f32> {
        &self.transform.get_rotation()
    }

    pub fn set_translation(&mut self, translation: &Vector3<f32>) {
        self.transform.set_translation(translation);
    }

    pub fn set_rotation(&mut self, rotation: &Quaternion<f32>) {
        self.transform.set_rotation(rotation);
    }

    pub fn translate(&mut self, translation: &Vector3<f32>) {
        self.transform.translate(translation);
    }

    pub fn get_view_matrix(&mut self) -> &Matrix4<f32> {
        &self.transform.get_matrix(true)
    }

    pub fn get_fov(&self) -> f32 {
        self.fov
    }

    pub fn set_fov(&mut self, fov: f32) {
        self.fov = fov;
        self.proj_dirty = true;
    }

    pub fn get_near(&self) -> f32 {
        self.near
    }

    pub fn set_near(&mut self, near: f32) {
        self.near = near;
        self.proj_dirty = true;
    }

    pub fn get_far(&self) -> f32 {
        self.far
    }

    pub fn set_far(&mut self, far: f32) {
        self.far = far;
        self.proj_dirty = true;
    }

    pub fn get_aspect_ratio(&self) -> Option<f32> {
        self.aspect_ratio
    }

    pub fn set_aspect_ratio(&mut self, aspect_ratio: Option<f32>) {
        self.aspect_ratio = aspect_ratio;
    }

    pub fn get_proj_matrix(&mut self) -> &Matrix4<f32> {
        if self.proj_dirty {
            let aspect_ratio = self.aspect_ratio.unwrap_or_else(|| -> f32 {
                let window = app().window();
                window.width() as f32 / window.height() as f32
            });
            
            self.proj_matrix = perspective(
                Deg(self.fov),
                aspect_ratio,
                self.near,
                self.far
            );
        }

        &self.proj_matrix
    }
}