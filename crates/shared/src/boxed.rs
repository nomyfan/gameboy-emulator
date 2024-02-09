use std::ops::{Deref, DerefMut};

use crate::boxed_array_fn;

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

impl<T: Clone, const COLS: usize, const ROWS: usize> Clone for BoxedMatrix<T, COLS, ROWS> {
    fn clone(&self) -> Self {
        Self(boxed_array_fn(|i| self[i].clone()))
    }
}

#[derive(Debug)]
pub struct BoxedArray<T, const SIZE: usize>(Box<[T; SIZE]>);

impl<T: Default, const SIZE: usize> Default for BoxedArray<T, SIZE> {
    fn default() -> Self {
        Self(boxed_array_fn(|_| T::default()))
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

impl<T: Clone, const SIZE: usize> Clone for BoxedArray<T, SIZE> {
    fn clone(&self) -> Self {
        Self(boxed_array_fn(|i| self[i].clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn array_clone() {
        let mut value: BoxedArray<u8, 2> = BoxedArray::default();
        let value_clone = value.clone();
        assert_eq!(value.as_ref(), value_clone.as_ref());

        value[0] = 12;
        assert_ne!(value.as_ref(), value_clone.as_ref());
    }

    #[test]
    fn array_from() {
        let value = [1, 2];
        let value = BoxedArray::from(&value);
        assert_eq!(value.as_ref(), &[1, 2]);
    }

    #[test]
    fn matrix_clone() {
        let mut value: BoxedMatrix<u8, 2, 2> = BoxedMatrix::default();
        let value_clone = value.clone();
        assert_eq!(value.as_ref(), value_clone.as_ref());

        value[0][0] = 12;
        assert_ne!(value.as_ref(), value_clone.as_ref());
    }
}
