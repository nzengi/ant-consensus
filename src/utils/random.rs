use rand::Rng;

/// Generate a random integer in range [min, max]
pub fn random_int(min: i32, max: i32) -> i32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..=max)
}

/// Generate a random float in range [min, max]
pub fn random_float(min: f64, max: f64) -> f64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..=max)
}

/// Generate random bytes
pub fn random_bytes(len: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..len).map(|_| rng.gen()).collect()
}

