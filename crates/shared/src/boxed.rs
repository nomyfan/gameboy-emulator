use std::ops::{Deref, DerefMut};

use crate::{boxed_array, boxed_array_fn};

pub struct BoxedMatrix<T, const COLS: usize, const ROWS: usize>(Box<[Box<[T; COLS]>; ROWS]>);

impl<T: Default, const COLS: usize, const ROWS: usize> Default for BoxedMatrix<T, COLS, ROWS> {
    fn default() -> Self {
        Self(boxed_array_fn(|_| boxed_array_fn(|_| T::default())))
    }
}

impl<T, const COLS: usize, const ROWS: usize> Deref for BoxedMatrix<T, COLS, ROWS> {
    type Target = Box<[Box<[T; COLS]>; ROWS]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, const COLS: usize, const ROWS: usize> DerefMut for BoxedMatrix<T, COLS, ROWS> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct BoxedArray<T, const SIZE: usize>(Box<[T; SIZE]>);

impl<T: Copy + Default, const SIZE: usize> Default for BoxedArray<T, SIZE> {
    fn default() -> Self {
        Self(boxed_array(T::default()))
    }
}

impl<T, const SIZE: usize> Deref for BoxedArray<T, SIZE> {
    type Target = Box<[T; SIZE]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, const SIZE: usize> DerefMut for BoxedArray<T, SIZE> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
