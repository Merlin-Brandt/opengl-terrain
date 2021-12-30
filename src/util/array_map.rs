
use std::mem;
use std::ops::FnOnce;

pub trait MappableArray<T> {
    type Output;
    fn map(self) -> Self::Output;
}

pub struct Array2Map<T>([T; 2]);

impl<T> MappableArray<T> for [T; 2] {
    type Output = Array2Map<T>;

    fn map(self) -> Array2Map<T> {
        Array2Map(self)
    }
}

impl<T> Array2Map<T> {
    pub fn with<F, R>(self, mut f: F) -> [R; 2] where F: FnMut(T) -> R {
        let mut arr = self.0;

        let arr0 = mem::replace(&mut arr[0], unsafe { mem::zeroed() });
        let arr1 = mem::replace(&mut arr[1], unsafe { mem::zeroed() });

        mem::forget(arr);

        [
            f(arr0),
            f(arr1),
        ]
    }
}
