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
        //first, convert to lowercase
        let text = text.to_lowercase();

        //convert into leetspeak
        let mut chars_vec: Vec<char> = text.chars().collect(); //turn into vec
        let char_num = chars_vec.len(); 
        for i in 0..char_num {
            let c = chars_vec[i];
            for j in 0..self.substitutions.len() {
                if c == self.substitutions[j].0 {
                    chars_vec[i] = self.substitutions[j].1; //if there is a match with the char and the first element of substitutions, replace
                    break;
                }
            }
        }

        //separator and space removal
        chars_vec.retain(|c| c.is_alphanumeric() || c.is_whitespace()); //https://doc.rust-lang.org/std/string/struct.String.html#method.retain
        
        //removing 3 or more repeats
        //concept: go through each char in char_vec. If the current char is the same as the previous, add to the counter. if not, change previous to current
        //if the counter is less than 3, add the char to the new string. if not, skip it
        let mut remove_repeats = String::new();
        let mut prev_char = ' ';
        let mut count = 0;

        for i in 0..chars_vec.len() {
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
        return remove_repeats;
    }
    

    pub fn normalize_preserving_spaces(&self, text: &str) -> String {
        //first, convert to lowercase
        let text = text.to_lowercase();

        //convert into leetspeak
        let mut chars_vec: Vec<char> = text.chars().collect(); //turn into vec
        let char_num = chars_vec.len(); 
        for i in 0..char_num {
            let c = chars_vec[i];
            for j in 0..self.substitutions.len() {
                if c == self.substitutions[j].0 {
                    chars_vec[i] = self.substitutions[j].1; //if there is a match with the char and the first element of substitutions, replace
                    break;
                }
            }
        }
        //separator and space removal, basically same as normalize but take out  || c.is_whitespace()
        chars_vec.retain(|c| c.is_alphanumeric()); //https://doc.rust-lang.org/std/string/struct.String.html#method.retain
        
        //removing 3 or more repeats
        //concept: go through each char in char_vec. If the current char is the same as the previous, add to the counter. if not, change previous to current
        //if the counter is less than 3, add the char to the new string. if not, skip it
        let mut remove_repeats = String::new();
        let mut prev_char = ' ';
        let mut count = 0;

        for i in 0..chars_vec.len() {
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
        return remove_repeats;
    }
}


/* ── SMOKE TEST ──
let n = Normalizer::new();
assert_eq!(n.normalize("g@g0"), "gago");
assert_eq!(n.normalize("gaaaago"), "gaago");
println!("Normalization smoke test passed done.");
*/
impl Default for Normalizer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normalize_leet_speak() {
        let n = Normalizer::new();
        assert_eq!(n.normalize("g@g0"), "gago");
        assert_eq!(n.normalize("b0b0"), "bobo");
        assert_eq!(n.normalize("t@ng1n@"), "tangina");
    }

    #[test]
    fn normalize_separators() {
        let n = Normalizer::new();
        assert_eq!(n.normalize("g.a.g.o"), "gago");
        assert_eq!(n.normalize("g-a-g-o"), "gago");
        assert_eq!(n.normalize("g_a_g_o"), "gago");
    }

    #[test]
    fn normalize_repeated_chars() {
        let n = Normalizer::new();
        assert_eq!(n.normalize("gaaaago"), "gaago");
        assert_eq!(n.normalize("boooobo"), "bobo");
    }

    #[test]
    fn normalize_preserving_spaces() {
        let n = Normalizer::new();
        assert_eq!(n.normalize_preserving_spaces("G@g0  m0!!"), "gago mo");
    }

    #[test]
    fn normalize_mixed_case() {
        let n = Normalizer::new();
        assert_eq!(n.normalize("GAGO"), "gago");
        assert_eq!(n.normalize("GaGo"), "gago");
    }

    #[test]
    fn normalize_empty_and_whitespace() {
        let n = Normalizer::new();
        assert_eq!(n.normalize(""), "");
        assert_eq!(n.normalize_preserving_spaces("   "), "");
    }
    // TODO: Write your own unit tests.
    // The integration test suite will verify correctness.
}
