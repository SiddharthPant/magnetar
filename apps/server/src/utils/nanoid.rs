use nanoid::nanoid;
use rand::{rngs::StdRng, RngExt, SeedableRng};

#[must_use]
pub fn prefixed_nanoid(prefix: &str, length: usize, is_seed_fixed: bool) -> String {
    let alphabet: [char; 58] = [
        '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J',
        'K', 'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c',
        'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v',
        'w', 'x', 'y', 'z',
    ];
    let id: String = if is_seed_fixed {
        let mut rng = StdRng::seed_from_u64(108);
        nanoid!(length, &alphabet, |size| {
            let mut bytes = vec![0u8; size];
            rng.fill(&mut bytes[..]);
            bytes
        })
    } else {
        nanoid!(length, &alphabet)
    };

    format!("{prefix}_{id}")
}
