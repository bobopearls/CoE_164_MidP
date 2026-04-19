use crate::error::MuraResult;

#[derive(Debug, Clone)]
pub struct Normalizer {
    substitutions: Vec<(char, char)>,
}

impl Normalizer {
    pub fn new() -> Self {
        let mut substitutions = Vec::new(); //substitutions defined above
        //list from slides
        substitutions.push(('@', 'a'));
        substitutions.push(('4', 'a'));
        substitutions.push(('3', 'e'));
        substitutions.push(('$', 's'));
        substitutions.push(('1', 'i'));
        substitutions.push(('7', 't'));
        substitutions.push(('0', 'o'));
        substitutions.push(('5', 's'));
        substitutions.push(('+', 't'));
        
        Self {substitutions}
    }

    pub fn normalize(&self, text: &str) -> String {
        //convert first into leetspeak
        let mut chars_vec: Vec<char> = text.chars().collect();
        let char_num = chars_vec.len();
        for i in 0..char_num {
            let c = chars_vec[i];
            for j in 0..self.substitutions.len() {
                if c == self.substitutions[j].0 {
                    chars_vec[i] = self.substitutions[j].1;
                    break;
                }
            }
        }

        let mut remove_repeats = String::new();
        let mut prev_char = ' ';
        let mut count = 0;

        for i in 0..char_num {
            let c = chars_vec[i];
            if c == prev_char {
                count += 1;
            } else {
                count = 1;
                prev_char = c;
            }
            if count < 3 {
                remove_repeats.push(c);
            }
        }

        //normalize 
        let mut input2 = remove_repeats;
        input2.retain(|c| c.is_alphanumeric()); //https://doc.rust-lang.org/std/string/struct.String.html#method.retain
        let input2 = input2.to_lowercase();
        return input2
        }
    }

    pub fn normalize_preserving_spaces(&self, text: &str) -> String {
        //convert first into leetspeak
        let mut chars_vec: Vec<char> = text.chars().collect();
        let char_num = chars_vec.len();
        for i in 0..char_num {
            let c = chars_vec[i];
            for j in 0..self.substitutions.len() {
                if c == self.substitutions[j].0 {
                    chars_vec[i] = self.substitutions[j].1;
                    break;
                }
            }
        }

        let mut remove_repeats = String::new();
        let mut prev_char = ' ';
        let mut count = 0;

        for i in 0..char_num {
            let c = chars_vec[i];
            if c == prev_char {
                count += 1;
            } else {
                count = 1;
                prev_char = c;
            }
            if count < 3 {
                remove_repeats.push(c);
            }
        }

        //normalize 
        let mut input2 = remove_repeats;
        input2.retain(|c| c.is_alphanumeric() || c.is_whitespace()); //https://doc.rust-lang.org/std/string/struct.String.html#method.retain
        let input2 = input2.to_lowercase();
        return input2
        }
    


impl Default for Normalizer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Write your own unit tests.
    // The integration test suite will verify correctness.
}
