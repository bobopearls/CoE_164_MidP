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
        if symdrome != 0{
            // the syndrome is the position of the flipped bit
            let error_pos = syndrome as usize;
            if error_pos < 1 || error_pos > 7{
                return Err(MuraError::Decode("Hamming syndrome out of range".into()));
            }
            // flip erroneous bit that converts the 1 indx to 0 indx
            bits[error_pos - 1] = !bits[error_pos - 1];
        }

        // Extract data bits from the correct codeword
        let d1_corr = bits[2] as u8;
        let d2_corr = bits[4] as u8;
        let d3_corr = bits[5] as u8;
        let d4_corr = bits[6] as u8;

        Ok((d1_corr << 3) | (d2_corr << 2) | (d3_corr << 1) | d4_corr)
    }
    
    pub fn encode(data: &[u8]) -> Vec<u8> {
        todo!()
    }

    pub fn decode(encoded: &[u8], original_len: usize) -> MuraResult<Vec<u8>> {
        todo!()
    }
}

pub fn crc32(data: &[u8]) -> u32 {
    todo!()
}

pub fn save_model(
    path: &Path,
    tokenizer: &BpeTokenizer,
    classifier: &LogisticClassifier,
) -> MuraResult<()> {
    todo!()
}

pub fn load_model(
    path: &Path,
) -> MuraResult<(BpeTokenizer, LogisticClassifier)> {
    todo!()
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
}
