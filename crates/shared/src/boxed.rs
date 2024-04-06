use std::ops::{Deref, DerefMut};

use crate::boxed_array_fn;

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
}
