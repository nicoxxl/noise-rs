use crate::noise_fns::NoiseFn;

/// Noise function that outputs the sum of the two output values from two source
/// functions.
pub struct Add<A, B>(A, B);

impl<A, B> Add<A, B> {
    pub fn new(source1: A, source2: B) -> Self {
        Self(source1, source2)
    }
}

impl<T, A, B> NoiseFn<T> for Add<A, B>
where
    T: Copy,
    A: NoiseFn<T>,
    B: NoiseFn<T>,
{
    fn get(&self, point: T) -> f64 {
        self.0.get(point) + self.1.get(point)
    }
}
