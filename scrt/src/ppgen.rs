use rand::seq::IndexedRandom;

const WORDS: &[&str] = &[
    "baby", "pikul", "ashy", "meedev", "hitler", "snowy", "raven", "orange", "blacky", "shadow",
    "oreo", "kitty", "luna", "ginger", "saffron", "mumu", "ocean", "metro", "sky", "road", "boat",
    "fish", "meat", "tree", "cloud", "pool", "storm", "rain", "forest", "desert", "sand", "cat",
];

pub fn generate_pass_phrase(words: usize) -> String {
    let mut rng = rand::rng();
    (0..words)
        .map(|_| WORDS.choose(&mut rng).unwrap())
        .cloned()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod test {

    #[test]
    fn test_pp_generation() {
        let pp = super::generate_pass_phrase(12);
        let words: Vec<_> = pp.split(' ').collect();
        assert_eq!(words.len(), 12)
    }
}
