use std::ops::{Deref, DerefMut};

use crate::{boxed_array, boxed_array_fn};

#[derive(Debug)]
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

impl<T: Copy, const COLS: usize, const ROWS: usize> Clone for BoxedMatrix<T, COLS, ROWS> {
    fn clone(&self) -> Self {
        Self(boxed_array_fn(|i| self[i].clone()))
    }
}

// impl<T, const COLS: usize, const ROWS: usize> From<[[T; COLS]; ROWS]>
//     for BoxedMatrix<T, COLS, ROWS>
// {
//     fn from(value: [[T; COLS]; ROWS]) -> Self {
//         let value: Vec<Box<[T; COLS]>> = value.into_iter().map(|row| Box::new(row)).collect();
//         let boxed_slice = value.into_boxed_slice();
//         let ptr = Box::into_raw(boxed_slice) as *mut [Box<[T; COLS]>; ROWS];
//         unsafe { Self(Box::from_raw(ptr)) }
//     }
// }

#[derive(Debug)]
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

impl<T: Copy, const SIZE: usize> From<&[T; SIZE]> for BoxedArray<T, SIZE> {
    fn from(value: &[T; SIZE]) -> Self {
        BoxedArray(boxed_array_fn(|i| value[i]))
    }
}

impl<T: Copy, const SIZE: usize> Clone for BoxedArray<T, SIZE> {
    fn clone(&self) -> Self {
        Self(boxed_array_fn(|i| self[i]))
    }
}
