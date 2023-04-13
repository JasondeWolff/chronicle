pub mod timer;
pub use timer::*;
pub mod rc_cell;
pub use rc_cell::*;

pub fn vec_remove_multiple<T>(vec: &mut Vec<T>, indices: &mut Vec<usize>) {
    indices.sort();    

    let mut j: usize = 0;
    for i in indices.iter() {
        vec.remove(i - j);
        j += 1;
    }
}