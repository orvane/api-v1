use rand::{thread_rng, Rng};

pub fn generate_random_code(length: usize) -> String {
    let mut rng = thread_rng();

    let code: String = (0..length)
        .map(|_| rng.gen_range(0..10).to_string())
        .collect();

    code
}
