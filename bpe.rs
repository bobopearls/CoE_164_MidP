use std::collections::HashMap;
use crate::error::MuraResult;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeRule {
    pub pair_a: u32,
    pub pair_b: u32,
    pub result: u32,
    pub priority: u32,
}

#[derive(Debug, Clone)]
pub struct BpeTokenizer {
    vocabulary: Vec<Vec<u8>>,
    merge_rules: Vec<MergeRule>,
    token_to_id: HashMap<Vec<u8>, u32>,
}

impl BpeTokenizer {
    pub fn new_byte_level() -> Self {
        let mut vocabulary: Vec<Vec<u8>> = Vec::with_capacity(256); //define the vocabulary vector, with_capacity used just to make sure all tokens are taken into account, and to allocate just enough mmemory
        let mut token_to_id: HashMap<Vec<u8>, u32> = HashMap::with_capacity(256); //same idea for the with_capcity as vocabulary

        for i in 0..256 { //to account for all bytes
            let byte_for_char = vec![i as u8]; //make sure it is u8 because a byte is 8 bits
            vocabulary.push(byte_for_char.clone()); //populate the vocabulary with the bytes for each char
            token_to_id.insert(byte_for_char, i as u32); //vector as key, token number as value. for chars with byte 0-255, it is equal
        }
        Self { 
            vocabulary, 
            merge_rules: Vec::new(), 
            token_to_id,
        }
    }

    pub fn vocab_size(&self) -> usize { self.vocabulary.len() }
    pub fn merge_rules(&self) -> &[MergeRule] { &self.merge_rules }
    pub fn vocabulary(&self) -> &[Vec<u8>] { &self.vocabulary }
    pub fn token_to_id(&self) -> &HashMap<Vec<u8>, u32> { &self.token_to_id }

    pub fn train(corpus: &str, vocab_size: usize) -> MuraResult<Self> {
        let mut chars_vec: Vec<u32> = corpus.bytes().map(|b| b as u32).collect(); //make corpus a vector of bytes 
        let mut bpetokenizer = Self::new_byte_level(); //new instance of bpetokenizer
        let mut new_token = 0; //for newly merged tokens

        while bpetokenizer.vocabulary.len() < vocab_size  { 
            let mut pairs_vec: Vec<(u32, u32)> = Vec::new(); //for the adjacent chars
            for i in 0..chars_vec.len() - 1 { //if -1 is not used, the last char will have the pair of nothing
                let pair = (chars_vec[i], chars_vec[i + 1]); 
                pairs_vec.push(pair);
            }

            let mut pair_map: HashMap<(u32, u32), u32> = HashMap::new(); //for counting the frequency of each pair
            for pair in pairs_vec { //enter each pair into hashmap and increment counter on duplicates
                    pair_map.entry(pair).and_modify(|counter| *counter += 1).or_insert(1); //https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.entry
                }
            if pair_map.is_empty() { //if there are no pairs left
                break;
            }
            
            let mut sorted_pairs: Vec<((u32, u32), u32)> = pair_map.clone().into_iter().collect(); //hashmap to vector for easier manipulation
            sorted_pairs.sort_by(|a, b| {b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0))}); //sort by frequency, then  lexicographically if same freq https://stackoverflow.com/questions/34555837/sort-hashmap-data-by-value
            
            let token_value = 256 + new_token; //has to be after 255 so it wont overlap with normal char tokens
            let current_high_pair = sorted_pairs[0].0; //get the highest frequency pair
            let pair_a = current_high_pair.0;
            let pair_b = current_high_pair.1;
            let result = token_value; //the new token will be the next available token value
            let priority = new_token + 1; //assign priority based on order of creation
            
            
            bpetokenizer.merge_rules.push(MergeRule { pair_a, pair_b, result, priority }); //add new merge rule for the highest freq pair
            let mut merged_bytes = bpetokenizer.vocabulary[pair_a as usize].clone(); //get bytes of paira
            let bytes_pair_b = bpetokenizer.vocabulary[pair_b as usize].clone(); //get bytes of pairb
            merged_bytes.extend(bytes_pair_b); //extend bytes of paira with that of pairb for the new token
            
            bpetokenizer.vocabulary.push(merged_bytes.clone());  //add the new token to vocab
            bpetokenizer.token_to_id.insert(merged_bytes, result); //add the new token to token_to_id

            let mut i = 0; //while loop with i was used instead of a for loop so that skipping over chars is possible and out of bounds wont happen
            let mut new_vec: Vec<u32> = Vec::new(); //for the new tokens after merging
            while i < chars_vec.len() { 
                if i < chars_vec.len() - 1 && (chars_vec[i], chars_vec[i + 1]) == (pair_a, pair_b) { //if the current pair is the highest freq pair, merge it into the new token
                    i += 2; //skip next char since it has been merged
                    new_vec.push(result); //add new token
                } else {
                    new_vec.push(chars_vec[i]); //keep current token
                    i += 1;
                }
            }
            pair_map.clear(); //clear pair_map for the next iteration
            chars_vec = new_vec; //update chars_vec with new tokens
            new_token += 1;
            
        }
        Ok(bpetokenizer)

    }


    pub fn encode(&self, text: &str) -> Vec<u32> {
        //more or less used the same structure in train()
        let mut all_tokens: Vec<u32> = text.bytes().map(|b| b as u32).collect();

        for rule in &self.merge_rules { //go through each merge rule
            //same reason as in train() for why a while loop was used instead of a for loop; to allow skipping over chars and to avoid out of bounds errors
            let mut i = 0;
            let mut new_tokens = Vec::new(); //store the new tokens after merging
            while i < all_tokens.len() {
                if i < all_tokens.len() - 1 && (all_tokens[i], all_tokens[i + 1]) == (rule.pair_a, rule.pair_b) {
                    new_tokens.push(rule.result); //merge into new token
                    i += 2; //skip the next token since it is merged
                } else {
                    new_tokens.push(all_tokens[i]);
                    i += 1;
                }
            }
            all_tokens = new_tokens;
        }
        all_tokens
    }

    pub fn decode(&self, tokens: &[u32]) -> MuraResult<String> {
        let chars_vec: Vec<u32> = tokens.to_vec(); //just to make sure it is a vec
        let mut final_chars_vec: Vec<u8> = Vec::new();
        for i in 0..chars_vec.len() {
            let c = chars_vec[i];
            let byte_list = &self.vocabulary[c as usize]; //get the bytes from the vocab for the current token
            for j in 0..byte_list.len() {
                final_chars_vec.push(byte_list[j]); //push into vec
            }
        }
        //change from vec to string, lossy prevents errors about invalid UTF-8 from happening 
        Ok(String::from_utf8_lossy(&final_chars_vec).into_owned()) //https://stackoverflow.com/questions/19076719/how-do-i-convert-a-vector-of-bytes-u8-to-a-string
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut to_byte_vec: Vec<u8> = Vec::new();

        to_byte_vec.extend(&(self.merge_rules.len() as u32).to_le_bytes()); //extend vec by number of merge rules

        for rule in &self.merge_rules {
            //convert each part of each rule in merge rule into bytes in little endian form, then extend the vec by it
            to_byte_vec.extend(&rule.pair_a.to_le_bytes()); 
            to_byte_vec.extend(&rule.pair_b.to_le_bytes());
            to_byte_vec.extend(&rule.result.to_le_bytes());
            to_byte_vec.extend(&rule.priority.to_le_bytes());
        }
        let total_vocab = self.vocabulary.len() as u32;
        to_byte_vec.extend(&total_vocab.to_le_bytes()); //extend by number of vocab words

        for byte_list in &self.vocabulary { 
            to_byte_vec.extend(&(byte_list.len() as u32).to_le_bytes()); //extend by length
            to_byte_vec.extend(byte_list);  //extend by bytes of the word itself
        }
        to_byte_vec
        }

   pub fn from_bytes(bytes: &[u8]) -> MuraResult<Self> {
        let mut cursor = 0;
        //https://docs.rs/bytes/latest/bytes/
        let rule_count_bytes = [bytes[cursor], bytes[cursor+1], bytes[cursor+2], bytes[cursor+3]]; //get the first 4 bytes for the merge rules
        let rule_count = u32::from_le_bytes(rule_count_bytes); //convert to u32
        cursor += 4; //plus 4 to go over the 4 bytes just used

        let mut merge_rules = Vec::new();

        for _i in 0..rule_count { //go through all merge rules and get all data for each 
            //pair_a
            let pair_a_bytes = [bytes[cursor], bytes[cursor+1], bytes[cursor+2], bytes[cursor+3]];
            let pair_a = u32::from_le_bytes(pair_a_bytes);
            cursor += 4; //same idea with the plus 4

            //pair_b
            let pair_b_bytes = [bytes[cursor], bytes[cursor+1], bytes[cursor+2], bytes[cursor+3]];
            let pair_b = u32::from_le_bytes(pair_b_bytes);
            cursor += 4;

            //result
            let result_bytes = [bytes[cursor], bytes[cursor+1], bytes[cursor+2], bytes[cursor+3]];
            let result = u32::from_le_bytes(result_bytes);
            cursor += 4;

            //priority
            let priority_bytes = [bytes[cursor], bytes[cursor+1], bytes[cursor+2], bytes[cursor+3]];
            let priority = u32::from_le_bytes(priority_bytes);
            cursor += 4;

            let rule = MergeRule { pair_a, pair_b, result, priority }; //putting data into a rule
            merge_rules.push(rule); //combine rules
        }

        let vocab_count_bytes = [bytes[cursor], bytes[cursor+1], bytes[cursor+2], bytes[cursor+3]]; //get the 4 bytes for vocab
        let vocab_count = u32::from_le_bytes(vocab_count_bytes); //convert to u32
        cursor += 4;

        let mut vocabulary = Vec::new();

        for _i in 0..vocab_count { //same idea as merge rules, go through each vocab word
            //same concept as each data in merge rules
            let len_bytes = [bytes[cursor], bytes[cursor+1], bytes[cursor+2], bytes[cursor+3]];
            let word_length = u32::from_le_bytes(len_bytes) as usize;
            cursor += 4;

            let mut word_size_bytes = Vec::new();
            for _j in 0..word_length {
                word_size_bytes.push(bytes[cursor]); //get size of each word in bytes, the combine into a vector
                cursor += 1;
            }
            
            vocabulary.push(word_size_bytes);
        }

        //remaking the hashmap for the token_to_id
        let mut token_to_id = HashMap::new();
        for i in 0..vocabulary.len() { //iterate through all words in vocab to be added
            let key = vocabulary[i].clone();
            let value = i as u32;
            token_to_id.insert(key, value);
        }

        Ok(Self {
            vocabulary,
            merge_rules,
            token_to_id,
            })
        }
}

#[cfg(test)]
mod tests {
    use super::*;

    //integration tests
    #[test]
    fn bpe_byte_level_has_256_tokens() {
        let t = BpeTokenizer::new_byte_level();
        assert_eq!(t.vocab_size(), 256);
    }

    #[test]
    fn bpe_train_increases_vocab() {
        let t = BpeTokenizer::train("aaabdaaabac", 260).unwrap();
        assert!(t.vocab_size() > 256);
        assert!(t.vocab_size() <= 260);
    }

    #[test]
    fn bpe_encode_decode_roundtrip() {
        let corpus = "hello hello world hello world";
        let t = BpeTokenizer::train(corpus, 280).unwrap();
        for text in &["hello", "world", "hello world", ""] {
            let decoded = t.decode(&t.encode(text)).unwrap();
            assert_eq!(*text, decoded.as_str());
        }
    }

    #[test]
    fn bpe_reject_empty_corpus() {
        assert!(BpeTokenizer::train("", 260).is_err());
    }

    #[test]
    fn bpe_reject_small_vocab() {
        assert!(BpeTokenizer::train("hello", 100).is_err());
    }

    #[test]
    fn bpe_serialization_roundtrip() {
        let t = BpeTokenizer::train("aaabdaaabac", 262).unwrap();
        let bytes = t.to_bytes();
        let t2 = BpeTokenizer::from_bytes(&bytes).unwrap();
        assert_eq!(t.vocab_size(), t2.vocab_size());
        assert_eq!(t.encode("aaab"), t2.encode("aaab"));
    }
    
    //unit tests
    #[test]
    fn train_test() {
        let corpus = "gagagogogago"; //should be two tokens "ga" and "go"
        let t = BpeTokenizer::train(corpus, 258).unwrap(); //two new vocab
        assert_eq!(t.vocab_size(), 258);
        assert_eq!(t.merge_rules().len(), 2); //since 2 new tokens were added, there should be 2 merge rules
    }

    #[test]
    fn encode_test() {
        let corpus = "gagogagogago";
        let t = BpeTokenizer::train(corpus, 258).unwrap();
        let encoded = t.encode("gago");
        assert_eq!(encoded.len(), 2); //2 is the length
    }

    #[test]
    fn encode_then_decode_test() {
        let corpus = "safe word safe";
        let t = BpeTokenizer::train(corpus, 259).unwrap();
        
        let og_input = "safe";
        let val = t.encode(og_input);
        let decoded_output = t.decode(&val).unwrap(); //decode the encoded value

        assert_eq!(og_input, decoded_output);
    }

    
}
