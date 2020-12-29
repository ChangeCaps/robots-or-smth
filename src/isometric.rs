use bevy::math::{Mat2, Mat3};

lazy_static::lazy_static! {
    pub static ref ISO_TO_SCREEN: Mat3 = Mat3::from_cols_array(&[
        //x,   y,   z
        32.0, 32.0, 0.0,
        -16.0, 16.0, 16.0,
        0.0, 0.0, 0.0,
    ]).transpose();

    pub static ref SCREEN_TO_ISO: Mat2 = Mat2::from_cols_array(&[
        //x,   y
        32.0, 32.0,
        -16.0, 16.0,
    ]).transpose().inverse();
}
