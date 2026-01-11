use crate::api;
pub use fastrand::Rng;

/// Returns an [Rng] with the given seed.
pub fn seed_rng(seed: u64) -> Rng {
    Rng::with_seed(seed)
}

/// Returns an [Rng] with the current time in nanoseconds as seed.
pub fn rng() -> Rng {
    seed_rng(api::time().read_local().nanosecond() as u64)
}
