use crate::noise_fns::NoiseFn;

/// Noise function that outputs the absolute value of the output value from the
/// source function.
pub struct Abs<S> {
    pub source: S,
}

impl<S> Abs<S> {
    pub fn new(source: S) -> Self {
        Self { source }
    }
}

impl<S, T> NoiseFn<T> for Abs<S>
where
    T: Copy,
    S: NoiseFn<T>,
{
    fn get(&self, point: T) -> f64 {
        (self.source.get(point)).abs()
    }
}
