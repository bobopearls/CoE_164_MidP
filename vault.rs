use std::fs;
use std::path::Path;

use crate::bpe::BpeTokenizer;
use crate::classifier::LogisticClassifier;
use crate::error::{MuraError, MuraResult};

pub struct Hamming74;

impl Hamming74 {
    // ── PROVIDED helpers ──
    pub fn bits_to_bytes(bits: &[bool]) -> Vec<u8> {
        let mut bytes = Vec::with_capacity((bits.len() + 7) / 8);
        let mut current = 0u8;
        let mut count = 0u8;
        for &bit in bits {
            current = (current << 1) | (bit as u8);
            count += 1;
            if count == 8 {
                bytes.push(current);
                current = 0;
                count = 0;
            }
        }
        if count > 0 {
            bytes.push(current << (8 - count));
        }
        bytes
    }

    pub fn bytes_to_bits(bytes: &[u8]) -> Vec<bool> {
        let mut bits = Vec::with_capacity(bytes.len() * 8);
        for &byte in bytes {
            for i in (0..8).rev() {
                bits.push((byte >> i) & 1 == 1);
            }
        }
        bits
    }
    // 4 bit nibble to a 7 nit hamming code word
    // 4 data bits and then 3 parity bits
    // ENCODE: compute each parity as an XOR fo the data bits
    // DECODE: recompute the parity checks to get syndrome of 0 (no error)
    //
    // layout of the bits: p1 p2 p3 d1 p3 d2 d3 d4 p is parity, d is data
    // parity coverage: 
    // p1 1,3,5,7 so d1 xor d2 xor d4
    // p2 2,3,6,7 so d1 xor d3 xor d4
    // p3 4,5,6,7 so d2 xor d3 xor d4
    pub fn encode_nibble(nibble: u8) -> [bool; 7] {
        let d1 = (nibble >> 3) & 1 == 1;
        let d2 = (nibble >> 2) & 1 == 1;
        let d3 = (nibble >> 1) & 1 == 1;
        let d4 = (nibble >> 0) & 1 == 1;

        let p1 = d1 ^ d2 ^ d4;
        let p2 = d1 ^ d3 ^ d4;
        let p3 = d2 ^ d3 ^ d4;

        // Codeword order: [p1, p2, d1, p3, d2, d3, d4]
        [p1, p2, d1, p3, d2, d3, d4]
    }

    // Decide the nibble back from what we made then compute the syndrome to check the errors
    // need to return an error if there is a position that cannot be corrected
    pub fn decode_nibble(codeword: [bool; 7]) -> MuraResult<u8> {
        let [p1, p2, d1, p3, d2, d3, d4] = codeword;

        // reverse of the syndrome bits kanina
        let s1 = (p1 ^ d1 ^ d2 ^ d4) as u8;
        let s2 = (p2 ^ d1 ^ d3 ^ d4) as u8;
        let s3 = (p3 ^ d2 ^ d3 ^ d4) as u8;

        let syndrome = s1 | (s2 << 1) | (s3 << 2);

        let mut bits = [p1, p2, d1, p3, d2, d3, d4]; 

        // syndrome if its not zero means there is an error
        if syndrome != 0 {
            let error_pos = syndrome as usize;
            if error_pos < 1 || error_pos > 7 {
                return Err(MuraError::HammingError("Hamming out of range".into())); // btw .into() converst obj to a different type 
            }
            // Flip the erroneous bit (convert from 1-indexed to 0-indexed)
            bits[error_pos - 1] = !bits[error_pos - 1];
        }
        
        // Extract data bits from the correct codeword
        let d1_corr = bits[2] as u8;
        let d2_corr = bits[4] as u8;
        let d3_corr = bits[5] as u8;
        let d4_corr = bits[6] as u8;

        Ok((d1_corr << 3) | (d2_corr << 2) | (d3_corr << 1) | d4_corr)
    }
    
    // split each byte into two nibbles
    // raw bytes become hamming 
    // so upper and lower nibble split that you pass through the encode_niblle then add the parity bits
    // total of 14 bits after running through the encode_nibble function. which is why *14 the data_len
    // then after bits_to_bytes, it packs the bit vec into actual bytes w/ MSB first
    pub fn encode(data: &[u8]) -> Vec<u8> {
        let mut bits: Vec<bool> = Vec::with_capacity(data.len()*14);  
        for &byte in data{
            let upper_nibble = (byte >> 4) & 0x0F;
            let lower_nibble = byte & 0x0F;
            bits.extend_from_slice(&Self::encode_nibble(upper_nibble));
            bits.extend_from_slice(&Self::encode_nibble(lower_nibble));
        }
        Self::bits_to_bytes(&bits)
    }

    // just the reverse of encoding, so:
    // unpack the bytes input to bits, then make a stream of Vec so that it is flat 
    // 14 bits is one original byte since we added the parity bits to the encoded code
    // then decode each nibble
    // then after, combine it back to a byte 
    pub fn decode(encoded: &[u8], original_len: usize) -> MuraResult<Vec<u8>> {
        let bits = Self::bytes_to_bits(encoded);
        let mut result = Vec::with_capacity(original_len);
        for i in 0..original_len{
            let start = i * 14;
            if start + 14 > bits.len(){
                return Err(MuraError::HammingError("Hamming too short".into()));
            }

            // unpacking the Hamming, careful slicing to extract the data into u8
            let upper_codeword = [bits[start], bits[start+1], bits[start+2], bits[start+3], bits[start+4], bits[start+5], bits[start+6]];
            let lower_codeword = [bits[start+7], bits[start+8], bits[start+9], bits[start+10], bits[start+11], bits[start+12], bits[start+13]];
            
            let upper_nibble = Self::decode_nibble(upper_codeword)?;
            let lower_nibble = Self::decode_nibble(lower_codeword)?;
            
            result.push((upper_nibble << 4) | lower_nibble);
        }
        Ok(result)
    }
}

// string of letters and numbers generated to verify data integ
// init 0xFFFFFFFF
// IEEE 802.3 polynomial ? 
// XOR first w crc then shift right repeatedly 
// final XOR with 0xFFFFFFFF
// crc32 b"hello" 0x3610A686
// crc32 b""      0x00000000
pub fn crc32(data: &[u8]) -> u32 {
    // LOOK UP TABLE
    let table: [u32; 256] = {
        let mut t: [u32; 256] = [0u32; 256];
        for i in 0u32..256u32 {
            let mut crc = i;
            for _ in 0..8 {
                if crc & 1 == 1 {
                    crc = (crc >> 1) ^ 0xEDB88320;
                } else {
                    crc >>= 1;
                }
            }
            t[i as usize] = crc;
        }
        t
    };
    let mut crc = 0xFFFFFFFFu32; // this initializes the reg! where u set all the bits to 1
    for &byte in data {
        let index = ((crc ^ byte as u32) & 0xFF) as usize;
        crc = (crc >> 8) ^ table[index];
    }
    crc ^ 0xFFFFFFFF // flip
}
// File format constants
const MAGIC: &[u8; 4] = b"MURA";
const VERSION: u32 = 1;

// Save a trained model (tokenizer + classifier) to the .mura file
// LE = little endian
// File layout:
//   [0..3]   Magic bytes "MURA"
//   [4..7]   Version (u32 LE) = must be 1
//   [8..11]  CRC-32 of raw payload (u32 LE). before Hamming, checksum
//   [12..15] Original payload length in bytes (u32 LE), tells decoder how many bits to expect after repairing
//   [16..]   Hamming encoded payload, BPE and classifier data
//
// Raw payload layout:
//   [0..3]          BPE section length (u32 LE)
//   [4..4+bpe_len]  Serialized BpeTokenizer
//   [4+bpe_len..4+bpe_len+4] CLF section length (u32 LE)
//   [4+bpe_len+4..]  Serialized LogisticClassifier

pub fn save_model(
    path: &Path,
    tokenizer: &BpeTokenizer,
    classifier: &LogisticClassifier,
) -> MuraResult<()> {
    // Serialize the components, need to save a sequence of raw bytes
    let bpe_bytes = tokenizer.to_bytes();
    let clf_bytes = classifier.to_bytes();

    let bpe_len = bpe_bytes.len() as u32;
    let clf_len = clf_bytes.len() as u32;

    // Reference of to_le_bytes https://doc.rust-lang.org/std/primitive.f32.html#method.to_le_bytes (little endian)
    let mut payload: Vec<u8> = Vec::new();
    payload.extend_from_slice(&bpe_len.to_le_bytes());
    payload.extend_from_slice(&bpe_bytes);
    payload.extend_from_slice(&clf_len.to_le_bytes());
    payload.extend_from_slice(&clf_bytes);

    let checksum = crc32(&payload);

    let encoded_payload = Hamming74::encode(&payload); 

    let original_len = payload.len() as u32;
    let mut file_bytes: Vec<u8> = Vec::new();
    file_bytes.extend_from_slice(MAGIC);
    file_bytes.extend_from_slice(&VERSION.to_le_bytes());
    file_bytes.extend_from_slice(&checksum.to_le_bytes());
    file_bytes.extend_from_slice(&original_len.to_le_bytes());
    file_bytes.extend_from_slice(&encoded_payload);

    fs::write(path, &file_bytes)
        .map_err(|e| MuraError::Io(e))

}

// Load the trained model from .mura then verify the MAGIC bytes, version, Hamming decoded, and payload
// checks the CRC-32 stuff then need to deserialize the tokenizer and the classifier 
pub fn load_model(
    path: &Path,
) -> MuraResult<(BpeTokenizer, LogisticClassifier)> {
    let file_bytes = fs::read(path)
        .map_err(|e| MuraError::Io(e))?;

    // check header size
    if file_bytes.len() < 16 {
        return Err(MuraError::VaultError("File too short ".into()));
    }

    // check magic bytes
    if &file_bytes[0..4] != MAGIC {
        return Err(MuraError::VaultError("Invalid magic bytes".into()));
    }

    // check version
    let version = u32::from_le_bytes(file_bytes[4..8].try_into().map_err(|_| MuraError::VaultError("Invalid version bytes".into()))?);
    if version != VERSION {
        return Err(MuraError::VaultError(format!("Unsupported version: {}", version)));
    }

    // read CRC and original length from the header
    let stored_crc = u32::from_le_bytes(file_bytes[8..12].try_into().map_err(|_| MuraError::VaultError("Invalid CRC bytes".into()))?);
    let original_len = u32::from_le_bytes(
        file_bytes[12..16]
            .try_into()
            .map_err(|_| MuraError::VaultError("Invalid original length bytes".into()))?
    ) as usize;

    // hamming decode the payload
    let encoded_payload = &file_bytes[16..];
    let payload = Hamming74::decode(encoded_payload, original_len)?;

    // verify CRC
    let computed_crc = crc32(&payload);
    if computed_crc != stored_crc {
        return Err(MuraError::VaultError(
            format!("CRC mismatch: stored {:#010X}, computed {:#010X}", stored_crc, computed_crc)
        ));
    }

    // parse BPE section
    if payload.len() < 4 {
        return Err(MuraError::VaultError("Payload too short for BPE length".into()));
    }
    let bpe_len = u32::from_le_bytes(payload[0..4].try_into().unwrap()) as usize;
    let bpe_end = 4 + bpe_len;
    if payload.len() < bpe_end {
        return Err(MuraError::VaultError("Payload too short for BPE data".into()));
    }
    let tokenizer = BpeTokenizer::from_bytes(&payload[4..bpe_end])?;

    // parse Classifier section
    let clf_start = bpe_end;
    if payload.len() < clf_start + 4 {
        return Err(MuraError::VaultError("Payload too short for CLF length".into()));
    }
    let clf_len = u32::from_le_bytes(
        payload[clf_start..clf_start+4]
            .try_into()
            .map_err(|_| MuraError::VaultError("Invalid CLF length bytes".into()))?
    ) as usize;
    let clf_data_start = clf_start + 4;
    let clf_end = clf_data_start + clf_len;
    if payload.len() < clf_end {
        return Err(MuraError::VaultError("Payload too short for CLF data".into()));
    }
    let classifier = LogisticClassifier::from_bytes(&payload[clf_data_start..clf_end])?;

    Ok((tokenizer, classifier))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Provided: verify the bit-packing helpers work.
    #[test]
    fn test_bits_to_bytes_full() {
        let bits = vec![true, false, true, true, false, false, false, true];
        assert_eq!(Hamming74::bits_to_bytes(&bits), vec![0b1011_0001]);
    }

    #[test]
    fn test_bytes_to_bits_roundtrip() {
        let original: Vec<bool> = vec![
            true, false, true, true, false, false, false, true,
        ];
        let bytes = Hamming74::bits_to_bytes(&original);
        let recovered = Hamming74::bytes_to_bits(&bytes);
        assert_eq!(original, recovered);
    }

    // TODO: Write your own unit tests for Hamming, CRC-32, and vault save/load.
    // The integration test suite will verify correctness.

    // CRC-32 tests:
    #[test]
    fn crc32_test_hello_in(){
        assert_eq!(crc32(b"hello"), 0x3610A686);
    }
    #[test]
    fn crc32_test_empty_in(){
        assert_eq!(crc32(b""), 0x00000000);
    }

    // Hamming test:
    #[test]
    fn encodecode_nibble_test(){
        for nibble in 0u8..16{
            let encoded = Hamming74::encode_nibble(nibble);
            let decoded = Hamming74::decode_nibble(encoded).unwrap();
            assert_eq!(nibble, decoded, "Nibble {nibble} failed test");
        }
    }

    #[test]
    fn decode_nibble_correct_single_bit_err(){
        for nibble in 0u8..16{
            let mut codeword = Hamming74::encode_nibble(nibble);
            // need to flip each bit then verify if correct
            for bit_position in 0..7{
                codeword[bit_position] = !codeword[bit_position]; // flipping
                let decoded = Hamming74::decode_nibble(codeword).unwrap();
                assert_eq!(nibble,decoded, "Nibble {nibble}, err at {bit_position}");
                codeword[bit_position] = !codeword[bit_position]; // flip restore
            }
        }
    }

     #[test]
    fn encodecode_byte_test() {
        let data = b"hello, world!";
        let encoded = Hamming74::encode(data);
        let decoded = Hamming74::decode(&encoded, data.len()).unwrap();
        assert_eq!(data.as_slice(), decoded.as_slice());
    }

    #[test]
    fn hamming_encode_test() {
        let data = vec![0xABu8; 18]; // 18B given
        let encoded = Hamming74::encode(&data);
        let expected_bits = 18 * 14; // 252
        let expected_bytes = (expected_bits + 7) / 8; // 32
        assert_eq!(encoded.len(), expected_bytes);
    }

}
