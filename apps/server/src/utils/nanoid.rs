use nanoid::nanoid;
use rand::{RngExt, SeedableRng, rngs::StdRng};

const LENGTH: usize = 16;

#[must_use]
pub fn prefixed_nanoid(prefix: &str, seed_value: Option<u64>) -> String {
    let alphabet: [char; 58] = [
        '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J',
        'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c',
        'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v',
        'w', 'x', 'y', 'z',
    ];

    let id = seed_value.map_or_else(
        || nanoid!(LENGTH, &alphabet),
        |seed| {
            let mut rng = StdRng::seed_from_u64(seed);
            nanoid!(LENGTH, &alphabet, |size| {
                let mut bytes = vec![0u8; size];
                rng.fill(&mut bytes[..]);
                bytes
            })
        },
    );

    format!("{prefix}_{id}")
}
