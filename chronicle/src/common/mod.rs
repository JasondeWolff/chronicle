pub mod timer;
pub use timer::*;
pub mod rc_cell;
pub use rc_cell::*;
use cgmath::{Vector3, Quaternion, Matrix4, SquareMatrix, Vector4};

pub fn vec_remove_multiple<T>(vec: &mut Vec<T>, indices: &mut Vec<usize>) {
    indices.sort();    

    let mut j: usize = 0;
    for i in indices.iter() {
        vec.remove(i - j);
        j += 1;
    }
}

pub fn quaternion_to_matrix(quaternion: &Quaternion<f32>) -> Matrix4<f32> {
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

pub fn up() -> Vector3<f32> {
    Vector3::new(0.0, 1.0, 0.0)
}

pub fn right() -> Vector3<f32> {
    Vector3::new(1.0, 0.0, 0.0)
}

pub fn forward() -> Vector3<f32> {
    Vector3::new(0.0, 0.0, -1.0)
}