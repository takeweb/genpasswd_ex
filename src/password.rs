use rand::Rng;
use rand_distr::{Alphanumeric, Distribution};

pub fn generate_password(length: usize, include_symbols: bool) -> String {
    let mut rng = rand::rng();
    let mut password: Vec<char> = (0..length)
        .map(|_| Alphanumeric.sample(&mut rng) as char)
        .collect();

    if include_symbols {
        let symbols = "!@#$%^&*()_+=-[]{};:,.<>?";
        for c in password.iter_mut().take(3) {
            *c = symbols
                .chars()
                .nth(rng.random_range(0..symbols.len()))
                .unwrap();
        }
    }

    password.into_iter().collect()
}
