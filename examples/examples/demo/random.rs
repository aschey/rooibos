use rand::distributions::{Distribution, Uniform};
use rand::rngs::StdRng;
use rand::SeedableRng;

#[derive(Clone)]
pub(crate) struct RandomData<S>
where
    S: Iterator + Clone,
    S::Item: Clone,
{
    pub(crate) source: S,
    pub(crate) points: Vec<S::Item>,
    pub(crate) tick_rate: usize,
}

#[derive(Clone)]
pub(crate) struct RandomDistribution {
    distribution: Uniform<u64>,
    rng: StdRng,
}

impl RandomDistribution {
    pub fn new(lower: u64, upper: u64) -> RandomDistribution {
        RandomDistribution {
            distribution: Uniform::new(lower, upper),
            rng: rand::rngs::StdRng::from_entropy(),
        }
    }
}

impl Iterator for RandomDistribution {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        Some(self.distribution.sample(&mut self.rng))
    }
}

impl<S> RandomData<S>
where
    S: Iterator + Clone,
    S::Item: Clone,
{
    pub(crate) fn on_tick(&mut self) {
        self.points = self.points[self.tick_rate..].to_vec();
        self.points
            .extend(self.source.by_ref().take(self.tick_rate));
    }
}
