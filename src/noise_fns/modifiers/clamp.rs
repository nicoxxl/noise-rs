use crate::{math, noise_fns::NoiseFn};

/// Noise function that clamps the output value from the source function to a
/// range of values.
pub struct Clamp<S> {
    /// Outputs a value.
    pub source: S,

    /// Bound of the clamping range. Default is -1.0 to 1.0.
    pub bounds: (f64, f64),
}

impl<S> Clamp<S> {
    pub fn new(source: S) -> Self {
        Self {
            source,
            bounds: (-1.0, 1.0),
        }
    }

    pub fn set_lower_bound(self, lower_bound: f64) -> Self {
        Self {
            bounds: (lower_bound, self.bounds.1),
            ..self
        }
    }

    pub fn set_upper_bound(self, upper_bound: f64) -> Self {
        Self {
            bounds: (self.bounds.0, upper_bound),
            ..self
        }
    }

    pub fn set_bounds(self, lower_bound: f64, upper_bound: f64) -> Self {
        Self {
            bounds: (lower_bound, upper_bound),
            ..self
        }
    }
}

impl<S, T> NoiseFn<T> for Clamp<S>
where
    T: Copy,
    S: NoiseFn<T>,
{
    fn get(&self, point: T) -> f64 {
        math::clamp(self.source.get(point), self.bounds.0, self.bounds.1)
    }
}
