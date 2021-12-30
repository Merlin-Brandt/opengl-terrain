
use super::num::Integer;

/// Inner value is guaranteed not to be zero.
pub struct NonZero<T: Integer>(T);

impl<T: Integer> NonZero<T> {
    #[inline]
    pub fn new(val: T) -> Option<NonZero<T>> {
        if val != T::zero() {
            Some(NonZero(val))
        } else {
            None
        }
    }

    pub fn map<F, R>(self, f: F) -> NonZero<R> where F: FnOnce(T) -> R, R: Integer {
        NonZero(f(self.val()))
    }

    #[inline]
    pub fn get(&self) -> &T {
        &self.0
    }

    #[inline]
    pub fn unwrap(self) -> T {
        self.val()
    }

    #[inline]
    pub fn val(self) -> T {
        self.0
    }
}

pub trait EnsureNotZero {
    type Item: Integer;
    fn ensure_not_zero(self) -> NonZero<Self::Item>;
}

impl<T> EnsureNotZero for T where T: Integer {
    type Item = T;
    fn ensure_not_zero(self) -> NonZero<T> {
        NonZero::new(self).unwrap()
    }
}

impl<T: Integer + Clone> Clone for NonZero<T> {
    fn clone(&self) -> NonZero<T> {
        NonZero(self.get().clone())
    }
}

impl<T: Integer + Copy> Copy for NonZero<T> {}

impl<T: Integer> AsRef<T> for NonZero<T> {
    fn as_ref(&self) -> &T {
        self.get()
    }
}
