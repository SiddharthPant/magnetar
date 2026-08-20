use nanoid::nanoid;
use rand::{RngExt, SeedableRng, rngs::StdRng};

#[must_use]
pub fn prefixed_nanoid(prefix: &str) -> String {
    let mut rng = StdRng::seed_from_u64(42);

    let id = nanoid!(10, &nanoid::alphabet::SAFE, |size| {
        let mut bytes = vec![0u8; size];
        rng.fill(&mut bytes[..]);
        bytes
    });

    format!("{prefix}_{id}")
}
