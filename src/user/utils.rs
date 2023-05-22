use ed25519_compact::{KeyPair, PublicKey, Seed};
use jwt::{FromBase64, ToBase64};

pub fn generate_keys(password: &String, reg_no: &String) -> KeyPair {
    let mut seed = [0u8; Seed::BYTES];
    let mut comb_string = reg_no.clone();
    comb_string.push_str(password.clone().as_str());
    let comb_bytes = comb_string.as_bytes();

    for (index, pos) in seed.iter_mut().enumerate() {
        *pos = if let Some(val) = comb_bytes.get(index) {
            *val
        } else {
            49
        };
    }
    let seed = Seed::from_slice(&seed).unwrap();
    KeyPair::from_seed(seed)
}

pub fn pk_from_string(pk: &String) -> PublicKey {
    let pk_dec: Result<Vec<u8>, jwt::Error> = FromBase64::from_base64(&pk);
    let pk_conv = PublicKey::from_slice(&pk_dec.unwrap());
    pk_conv.unwrap()
}
