use cgmath::{Vector3, Vector4, Quaternion, Matrix4};
use cgmath::prelude::SquareMatrix;

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    translation: Vector3<f32>,
    rotation: Quaternion<f32>,
    scale: Vector3<f32>,

    model_matrix: Matrix4<f32>,
    model_matrix_inv_trans: Matrix4<f32>,
    model_matrix_dirty: bool
}

impl Transform {
    pub fn new() -> Self {
        Transform {
            translation: Vector3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
            model_matrix: SquareMatrix::identity(),
            model_matrix_inv_trans: SquareMatrix::identity(),
            model_matrix_dirty: true
        }
    }

    pub fn get_translation(&self) -> Vector3<f32> {
        self.translation
    }

    pub fn get_rotation(&self) -> Quaternion<f32> {
        self.rotation
    }

    pub fn get_scale(&self) -> Vector3<f32> {
        self.scale
    }

    pub fn set_translation(&mut self, translation: &Vector3<f32>) {
        self.translation = translation.clone();
        self.model_matrix_dirty = true;
    }

    pub fn set_rotation(&mut self, rotation: &Quaternion<f32>) {
        self.rotation = rotation.clone();
        self.model_matrix_dirty = true;
    }

    pub fn set_scale(&mut self, scale: &Vector3<f32>) {
        self.scale = scale.clone();
        self.model_matrix_dirty = true;
    }

    pub fn translate(&mut self, translation: &Vector3<f32>) {
        self.set_translation(&(self.translation + translation));
    }

    pub fn scale(&mut self, scale: &Vector3<f32>) {
        self.set_scale(&(self.scale + scale));
    }

    pub fn get_matrix(&mut self) -> Matrix4<f32> {
        if self.model_matrix_dirty {
            self.recalculate_matrix();
        }

        self.model_matrix
    }

    pub fn get_matrix_inv_trans(&mut self) -> Matrix4<f32> {
        if self.model_matrix_dirty {
            self.recalculate_matrix();
        }

        self.model_matrix_inv_trans
    }

    fn recalculate_matrix(&mut self) {
        self.model_matrix = Matrix4::from_translation(self.translation)
                                    * Self::quaternion_to_matrix(&self.rotation)
                                    * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);
        self.model_matrix_inv_trans = self.model_matrix.clone()
            .invert().unwrap();
        self.model_matrix_inv_trans.transpose_self();
        self.model_matrix_dirty = false;
    }

    fn quaternion_to_matrix(quaternion: &Quaternion<f32>) -> Matrix4<f32> {
        let mut mat4: Matrix4<f32> = SquareMatrix::identity();

        let quat = Vector4::<f32>::new(quaternion.v.x, quaternion.v.y, quaternion.v.z, quaternion.s);

        mat4.x[0 - 0] = 1.0 - 2.0 * quat.y * quat.y - 2.0 * quat.z * quat.z;
        mat4.x[1 - 0] = 2.0 * quat.x * quat.y - 2.0 * quat.w * quat.z;
        mat4.x[2 - 0] = 2.0 * quat.x * quat.z + 2.0 * quat.w * quat.y;
        mat4.y[4 - 4] = 2.0 * quat.x * quat.y + 2.0 * quat.w * quat.z;
        mat4.y[5 - 4] = 1.0 - 2.0 * quat.x * quat.x - 2.0 * quat.z * quat.z;
        mat4.y[6 - 4] = 2.0 * quat.y * quat.z - 2.0 * quat.w * quat.x;
        mat4.z[8 - 8] = 2.0 * quat.x * quat.z - 2.0 * quat.w * quat.y;
        mat4.z[9 - 8] = 2.0 * quat.y * quat.z + 2.0 * quat.w * quat.x;
        mat4.z[10- 8] = 1.0 - 2.0 * quat.x * quat.x - 2.0 * quat.y * quat.y;

        mat4
    }
}