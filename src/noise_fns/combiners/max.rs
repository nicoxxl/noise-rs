use crate::noise_fns::NoiseFn;

/// Noise function that outputs the smaller of the two output values from two source
/// functions.
pub struct Max<A, B>(A, B);

impl<A, B> Max<A, B> {
    pub fn new(source1: A, source2: B) -> Self {
        Self(source1, source2)
    }
}

impl<T, A, B> NoiseFn<T> for Max<A, B>
where
    T: Copy,
    A: NoiseFn<T>,
    B: NoiseFn<T>,
{
    fn get(&self, point: T) -> f64 {
        (self.0.get(point)).max(self.1.get(point))
    }
}
