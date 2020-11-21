use crate::noise_fns::NoiseFn;

/// Noise function that raises the output value from the first source function
/// to the power of the output value of the second source function.
pub struct Power<A, B>(A, B);

impl<A, B> Power<A, B> {
    pub fn new(source1: A, source2: B) -> Self {
        Self(source1, source2)
    }
}

impl<T, A, B> NoiseFn<T> for Power<A, B>
where
    T: Copy,
    A: NoiseFn<T>,
    B: NoiseFn<T>,
{
    fn get(&self, point: T) -> f64 {
        (self.0.get(point)).powf(self.1.get(point))
    }
}
