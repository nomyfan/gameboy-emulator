/// Immutable value builder.
/// Without introducing `mut` in the context.
pub trait ImBuilder {
    fn map<R>(self, transform: impl FnOnce(Self) -> R) -> R
    where
        Self: Sized,
    {
        transform(self)
    }

    fn if_then_fn(
        self,
        condition: impl FnOnce(&Self) -> bool,
        transform: impl FnOnce(Self) -> Self,
    ) -> Self
    where
        Self: Sized,
    {
        if condition(&self) {
            transform(self)
        } else {
            self
        }
    }

    fn if_then(self, condition: bool, transform: impl FnOnce(Self) -> Self) -> Self
    where
        Self: Sized,
    {
        if condition {
            transform(self)
        } else {
            self
        }
    }
}

impl<T> ImBuilder for T where T: Sized {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn if_then() {
        let value = 1.if_then_fn(|v| v == &1, |v| v + 1).if_then_fn(|v| v % 2 != 0, |v| v * 2);

        assert_eq!(value, 2);
    }

    #[test]
    fn map() {
        let value = 1.if_then(true, |v| v + 100).map(|v| v.to_string());

        assert_eq!(value, "101");
    }
}
