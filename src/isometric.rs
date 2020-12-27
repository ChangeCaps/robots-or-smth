use bevy::math::Mat2;

lazy_static::lazy_static! {
    pub static ref ISO_TO_SCREEN: Mat2 = Mat2::from_cols_array(&[
        32.0, 16.0,
        16.0, 32.0,
    ]);
}
