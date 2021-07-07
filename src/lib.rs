use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::{num::NonZeroU32};
use std::str;
use rand_core::{RngCore, OsRng};
use to_binary::BinaryString;
use std::path::Path;
use ring::{digest, pbkdf2};
use hex;

static PBKDF2_ALG: pbkdf2::Algorithm = pbkdf2::PBKDF2_HMAC_SHA512;
const CREDENTIAL_LEN: usize = digest::SHA512_OUTPUT_LEN;
const SUB_LEN: usize = 11;
const BYTES_LEN: usize = 16;
const BITS_PER_CHECKSUM_DIGIT: usize = 32;
const DEFAULT_PASSPHRASE: &str = "";
const DEFAULT_SALT_BASE: &str = "mnemonic";
const WORDLIST_PATH: &str = "./wordlist.txt";

pub type Credential = [u8; CREDENTIAL_LEN];

pub struct SeedBuilder<'a> {
    passphrase: &'a str,
    salt: Option<Vec<u8>>,
}

impl<'a> Default for SeedBuilder<'a> {
    fn default() -> Self {
        let salt = DEFAULT_SALT_BASE.to_string() + DEFAULT_PASSPHRASE;
        SeedBuilder {
            passphrase: DEFAULT_PASSPHRASE,
            salt: Some(salt.as_bytes().to_vec())
        }
    }
}

impl<'a> SeedBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn salt(mut self, salt: Vec<u8>) -> Self {
        self.salt.replace(salt);
        self
    }

    pub fn passphrase(mut self, passphrase: &'a str) -> Self {
        self.passphrase = passphrase;
        let salt = DEFAULT_SALT_BASE.to_string() + passphrase;
        self.salt.replace(salt.as_bytes().to_vec());
        self
    }

    pub fn build(self) -> Result<Seed, String> {
        let mut key = [0u8; BYTES_LEN];
        OsRng.fill_bytes(&mut key);

        let result = digest::digest(&digest::SHA256, &key);
        let BinaryString(b) = BinaryString::from(result.as_ref());

        let BinaryString(bin) = BinaryString::from(&key[..]);
        let checksum_digits = bin.len() / BITS_PER_CHECKSUM_DIGIT;
        let checksum = &b[..checksum_digits];
        let ent = bin + checksum;

        let subs = ent.as_bytes()
            .chunks(SUB_LEN)
            .map(str::from_utf8)
            .collect::<Result<Vec<&str>, _>>()
            .unwrap();

        let indices: Vec<usize> = subs.iter()
            .map(|b| {
                let intval = isize::from_str_radix(b, 2).unwrap();
                intval
            }).map(|i| {
                i as usize
            }).collect();

        let path = Path::new(WORDLIST_PATH);
        if !path.exists() {
            return Err(format!("wordlist file at {} does not exist", WORDLIST_PATH));
        }

        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        let words: Vec<String> = reader.lines().into_iter()
            .map(|o| o.unwrap())
            .collect();

        let mnemonic_words: Vec<String> = indices.iter().map(|i| {
            words[*i].to_owned()
        }).collect();

        let mut salt = (DEFAULT_SALT_BASE.to_string() + self.passphrase).as_bytes().to_vec();
        if let Some(s) = self.salt {
            salt = s;
        }

        let password = mnemonic_words.join(" ");
        let mut seed_store: Credential = [0u8; CREDENTIAL_LEN];
        let iterations = NonZeroU32::new(100_000).unwrap();
        pbkdf2::derive(PBKDF2_ALG, iterations, &salt,
                        password.as_bytes(), &mut seed_store);

        let hex_str = hex::encode(&seed_store[..]);

        Ok(Seed {
            mnemonic: mnemonic_words,
            hex: hex_str,
            entropy: seed_store,
        })
    }
}

pub struct Seed {
    pub mnemonic: Vec<String>,
    pub entropy: Credential,
    pub hex: String,
}

impl ToString for Seed {
    fn to_string(&self) -> String {
        if self.hex.len() <= 0 {
            return hex::encode(&self.entropy[..]);
        }
        self.hex.to_owned()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_building() -> Result<(), String> {
        let mut rand = [0u8; 16];
        OsRng.fill_bytes(&mut rand);

        let default_builder = SeedBuilder::new();
        let custom_builder = SeedBuilder::new()
            .passphrase("holymoly")
            .salt(rand[..].to_vec());

        assert_eq!(default_builder.passphrase, "");
        assert_eq!(default_builder.salt.to_owned().unwrap(), "mnemonic".as_bytes().to_vec());
        assert_eq!(custom_builder.passphrase, "holymoly");
        assert_eq!(custom_builder.salt.to_owned().unwrap(), rand[..].to_vec());

        let default_seed = default_builder.build()?;
        let custom_seed = custom_builder.build()?;

        assert_eq!(default_seed.mnemonic.len(), 12);
        assert_eq!(custom_seed.mnemonic.len(), 12);
        assert_eq!(default_seed.entropy.len(), 64);
        assert_eq!(custom_seed.entropy.len(), 64);
        Ok(())
    }
}