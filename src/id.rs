use sqids::Sqids;

// Use a static number to encode with the id for more entropy
const STATIC_SALT: u64 = 42;

pub fn generate_public_id(id: i32) -> String {
    let sqids = Sqids::builder()
        .min_length(4) // Ensure minimum length like commit SHA
        .build()
        .expect("Failed to create Sqids instance");

    // Encode the id with a static number for more entropy
    sqids
        .encode(&[STATIC_SALT, id as u64])
        .expect("Failed to generate public ID")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_id_generation() {
        let id1 = generate_public_id(1);
        let id2 = generate_public_id(2);
        let id3 = generate_public_id(100);

        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert!(id1.len() >= 4);
        assert!(id2.len() >= 4);
        assert!(id3.len() >= 4);
    }
}




