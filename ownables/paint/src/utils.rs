use crate::store::IdbStorage;
use cosmwasm_std::{Addr, Api, BlockInfo, CanonicalAddr, ContractInfo, Empty, Env, MemoryStorage, OwnedDeps, Querier, RecoverPubkeyError, StdError, StdResult, Timestamp, VerificationError};
use std::marker::PhantomData;
use blake2::Blake2bVar;
use blake2::digest::{Update, VariableOutput};
use sha2::Digest as Sha2Digest;
use crate::IdbStateDump;
use sha3::{Digest};

pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub fn create_lto_env() -> Env {
    Env {
        block: BlockInfo {
            height: 0,
            time: Timestamp::from_seconds(0),
            chain_id: "lto".to_string(),
        },
        contract: ContractInfo {
            address: Addr::unchecked(""),
        },
        transaction: None,
    }
}

pub fn load_lto_deps(state_dump: Option<IdbStateDump>) -> OwnedDeps<MemoryStorage, EmptyApi, EmptyQuerier, Empty> {
    match state_dump {
        None => OwnedDeps {
            storage: MemoryStorage::default(),
            api: EmptyApi::default(),
            querier: EmptyQuerier::default(),
            custom_query_type: PhantomData,
        },
        Some(dump) => {
            let idb_storage = IdbStorage::load(dump);
            OwnedDeps {
                storage: idb_storage.storage,
                api: EmptyApi::default(),
                querier: EmptyQuerier::default(),
                custom_query_type: PhantomData,
            }
        }
    }

}

/// takes a b58 of compressed secp256k1 pk
pub fn address_eip155(public_key: String) -> Result<Addr, StdError> {
    if public_key.is_empty() {
        return Err(StdError::not_found("empty input"));
    }

    // decode b58 pk
    let pk = bs58::decode(public_key.as_bytes()).into_vec();
    let decoded_pk = match pk {
        Ok(pk) => pk,
        Err(e) => return Err(StdError::generic_err(e.to_string())),
    };

    // instantiate secp256k1 public key from input
    let public_key = secp256k1::PublicKey::from_slice(decoded_pk.as_slice()).unwrap();
    let mut uncompressed_hex_pk = hex::encode(public_key.serialize_uncompressed());
    if uncompressed_hex_pk.starts_with("04") {
        uncompressed_hex_pk = uncompressed_hex_pk.split_off(2);
    }

    // pass the raw bytes to keccak256
    let uncompressed_raw_pk = hex::decode(uncompressed_hex_pk).unwrap();

    let mut hasher = sha3::Keccak256::new();
    hasher.input(uncompressed_raw_pk.as_slice());
    let hashed_addr = hex::encode(hasher.result().as_slice()).to_string();

    let result = &hashed_addr[hashed_addr.len() - 40..];
    let checksum_addr = "0x".to_owned() + eip_55_checksum(result).as_str();

    Ok(Addr::unchecked(checksum_addr))
}

fn eip_55_checksum(addr: &str) -> String {
    let mut checksum_hasher = sha3::Keccak256::new();
    checksum_hasher.input(&addr[addr.len() - 40..].as_bytes());
    let hashed_addr = hex::encode(checksum_hasher.result()).to_string();

    let mut checksum_buff = "".to_owned();
    let result_chars: Vec<char> = addr.chars()
        .into_iter()
        .collect();
    let keccak_chars: Vec<char> = hashed_addr.chars()
        .into_iter()
        .collect();
    for i in 0..addr.len() {
        let mut char = result_chars[i];
        if char.is_alphabetic() {
            let keccak_digit = keccak_chars[i]
                .to_digit(16)
                .unwrap();
            // if the corresponding hex digit >= 8, convert to uppercase
            if keccak_digit >= 8 {
                char = char.to_ascii_uppercase();
            }
        }
        checksum_buff += char.to_string().as_str();
    }

    checksum_buff
}

pub fn address_lto(network_id: char, public_key: String) -> Result<Addr, StdError> {
    if network_id != 'L' && network_id != 'T' {
        return Err(StdError::generic_err("unrecognized network_id"));
    }
    if bs58::decode(public_key.clone()).into_vec().is_err() {
        return Err(StdError::generic_err("invalid public key"));
    }

    // decode b58 of pubkey into byte array
    let public_key = bs58::decode(public_key).into_vec().unwrap();
    // get the ascii value from network char
    let network_id = network_id as u8;
    println!("secure hash {:?}", public_key);
    let pub_key_secure_hash = secure_hash(public_key.as_slice());
    println!("secure hash {:?}", pub_key_secure_hash);
    // get the first 20 bytes of the securehash
    let address_bytes = &pub_key_secure_hash[0..20];
    let version = &1_u8.to_be_bytes();
    let checksum_input:Vec<u8> = [version, &[network_id], address_bytes].concat();

    // checksum is the first 4 bytes of secureHash of version, chain_id, and hash
    let checksum = &secure_hash(checksum_input.as_slice())
        .to_vec()[0..4];

    let addr_fields = [
        version,
        &[network_id],
        address_bytes,
        checksum
    ];

    let address: Vec<u8> = addr_fields.concat();
    Ok(Addr::unchecked(base58(address.as_slice())))
}

fn base58(input: &[u8]) -> String {
    bs58::encode(input).into_string()
}

fn secure_hash(m: &[u8]) -> Vec<u8> {
    let mut hasher = Blake2bVar::new(32).unwrap();
    hasher.update(m);
    let mut buf = [0u8; 32];
    hasher.finalize_variable(&mut buf).unwrap();

    // get the sha256 of blake
    let mut sha256_hasher = sha2::Sha256::new();
    Update::update(&mut sha256_hasher, buf.as_slice());
    let res = sha256_hasher.finalize();
    // let mut hasher = sha2::Sha256::new();
    // hasher.update(&buf);
    // let mut buf = hasher.finalize();
    res.to_vec()
}

const CANONICAL_LENGTH: usize = 54;

/// Empty Querier that is meant to conform the traits expected by the cosmwasm standard contract syntax. It should not be used whatsoever
#[derive(Default)]
pub struct EmptyQuerier {}

impl Querier for EmptyQuerier {
    fn raw_query(&self, _bin_request: &[u8]) -> cosmwasm_std::QuerierResult {
        todo!()
    }
}

// EmptyApi that is meant to conform the traits by the cosmwasm standard contract syntax. The functions of this implementation are not meant to be used or produce any sensible results.
#[derive(Copy, Clone)]
pub struct EmptyApi {
    /// Length of canonical addresses created with this API. Contracts should not make any assumtions
    /// what this value is.
    canonical_length: usize,
}

impl Default for EmptyApi {
    fn default() -> Self {
        EmptyApi {
            canonical_length: CANONICAL_LENGTH,
        }
    }
}

impl Api for EmptyApi {
    fn addr_validate(&self, human: &str) -> StdResult<Addr> {
        self.addr_canonicalize(human).map(|_canonical| ())?;
        Ok(Addr::unchecked(human))
    }

    fn addr_canonicalize(&self, human: &str) -> StdResult<CanonicalAddr> {
        // Dummy input validation. This is more sophisticated for formats like bech32, where format and checksum are validated.
        if human.len() < 3 {
            return Err(StdError::generic_err(
                "Invalid input: human address too short",
            ));
        }
        if human.len() > self.canonical_length {
            return Err(StdError::generic_err(
                "Invalid input: human address too long",
            ));
        }

        let mut out = Vec::from(human);

        // pad to canonical length with NULL bytes
        out.resize(self.canonical_length, 0x00);
        // // content-dependent rotate followed by shuffle to destroy
        // // the most obvious structure (https://github.com/CosmWasm/cosmwasm/issues/552)
        // let rotate_by = digit_sum(&out) % self.canonical_length;
        // out.rotate_left(rotate_by);
        // for _ in 0..SHUFFLES_ENCODE {
        //     out = riffle_shuffle(&out);
        // }
        Ok(out.into())
    }

    fn addr_humanize(&self, canonical: &CanonicalAddr) -> StdResult<Addr> {
        if canonical.len() != self.canonical_length {
            return Err(StdError::generic_err(
                "Invalid input: canonical address length not correct",
            ));
        }

        let tmp: Vec<u8> = canonical.clone().into();
        // // Shuffle two more times which restored the original value (24 elements are back to original after 20 rounds)
        // for _ in 0..SHUFFLES_DECODE {
        //     tmp = riffle_shuffle(&tmp);
        // }
        // // Rotate back
        // let rotate_by = digit_sum(&tmp) % self.canonical_length;
        // tmp.rotate_right(rotate_by);
        // Remove NULL bytes (i.e. the padding)
        let trimmed = tmp.into_iter().filter(|&x| x != 0x00).collect();
        // decode UTF-8 bytes into string
        let human = String::from_utf8(trimmed)?;
        Ok(Addr::unchecked(human))
    }

    fn secp256k1_verify(
        &self,
        _message_hash: &[u8],
        _signature: &[u8],
        _public_key: &[u8],
    ) -> Result<bool, VerificationError> {
        Err(VerificationError::unknown_err(0))
    }

    fn secp256k1_recover_pubkey(
        &self,
        _message_hash: &[u8],
        _signature: &[u8],
        _recovery_param: u8,
    ) -> Result<Vec<u8>, RecoverPubkeyError> {
        Err(RecoverPubkeyError::unknown_err(0))
    }

    fn ed25519_verify(
        &self,
        _message: &[u8],
        _signature: &[u8],
        _public_key: &[u8],
    ) -> Result<bool, VerificationError> {
        Ok(true)
    }

    fn ed25519_batch_verify(
        &self,
        _messages: &[&[u8]],
        _signatures: &[&[u8]],
        _public_keys: &[&[u8]],
    ) -> Result<bool, VerificationError> {
        Ok(true)
    }

    fn debug(&self, message: &str) {
        println!("{}", message);
    }
}

#[allow(dead_code)]
fn generate_seed(mnemonic: &str, nonce: u32) -> Vec<u8> {
    let nonce_bytes = [
        ((nonce >> 24) & 0xff) as u8,
        ((nonce >> 16) & 0xff) as u8,
        ((nonce >> 8) & 0xff) as u8,
        ((nonce >> 0) & 0xff) as u8,
    ];
    let seed_input: Vec<u8> = [
        nonce_bytes.as_slice().to_vec(),
        mnemonic.as_bytes().to_vec()
    ].concat();

    secure_hash(seed_input.as_slice())
}

#[cfg(test)]
mod utils {
    use cosmwasm_std::StdError;
    use crate::utils::{address_eip155, address_lto,};

    #[test]
    fn test_derive_eip155_address() {
        let compressed_b58_pk = "v3KjemAaDRYztCiwdT9X72waHdpTq6tHBxyqqCBfFCf7";

        let result = address_eip155(compressed_b58_pk.to_string()).unwrap();

        assert_eq!(result.to_string(), "0x6464FD2B55cACE128748CB0c9889fD5E37787526");
    }

    #[test]
    fn test_eip155_empty_input() {
        let pub_key = "";
        let err = address_eip155(pub_key.to_string()).unwrap_err();

        assert!(matches!(err, StdError::NotFound {..}));
    }

    #[test]
    fn test_eip155_invalid_b58() {
        let pub_key = "v3KjemAaDRYztCiwdT9X70waHdpTq6tHBxyqqCBfFCf7";
        let err = address_eip155(pub_key.to_string()).unwrap_err();

        assert!(matches!(err, StdError::GenericErr {..}));
    }

    #[test]
    fn test_derive_lto_address() {
        let result = address_lto(
            'T',
            "v3KjemAaDRYztCiwdT9X72waHdpTq6tHBxyqqCBfFCf7".to_string(),
        ).unwrap();

        assert_eq!(result.to_string(), "3NBd71MErsjwmStnj8PQECHP1JL2jvuY2HW");
    }

    #[test]
    fn test_derive_lto_address_invalid_network_id() {
        let err = address_lto(
            'A',
            "GjSacB6a5DFNEHjDSmn724QsrRStKYzkahPH67wyrhAY".to_string(),
        ).unwrap_err();
        assert!(matches!(err, StdError::GenericErr { .. }));
    }

    #[test]
    fn test_derive_lto_address_invalid_pub_key() {
        let err = address_lto(
            'L',
            "GjSacB6a5Dl1iINEHjDSmnQsrRStKYzkahPH67wyrhAY".to_string(),
        ).unwrap_err();

        assert!(matches!(err, StdError::GenericErr { .. }));
    }
}