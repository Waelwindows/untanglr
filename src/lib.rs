extern crate regex;
use regex::Regex;
use std::cmp::max;
use std::collections::HashMap;
use std::str;

// The LanguageModel holds the path to the dictionary (english
// by default), the maximum word cost and the maximum word length
pub struct LanguageModel {
    word_cost: HashMap<String, f64>,
    max_wlen: usize,
}

impl LanguageModel {
    // Initialize the LanguageModel using the words in the dictionary to
    // get the maximum word length and calculate word cost uzing Zipf's law
    pub fn new() -> LanguageModel {
        let dict = include_str!("dicts/english.txt");

        let word_count: f64 = dict.len() as f64;
        let mut max_wlen: usize = 0;
        let mut word_cost: HashMap<String, f64> = HashMap::new();

        for (index, line) in dict.lines().enumerate() {
            let word = line; // Ignore errors.
            let word_len: usize = word.chars().count();
            max_wlen = max(word_len, max_wlen);

            word_cost.insert(
                String::from(word),
                ((index + 1) as f64 * word_count.ln()).ln(),
            );
        }

        LanguageModel {
            word_cost,
            max_wlen,
        }
    }

    pub fn untangle(&self, s: &str) -> Vec<String> {
        let re = Regex::new(r"[^a-zA-Z0-9']+").unwrap();
        re.split(s).flat_map(|x| self.split(x.into())).collect()
    }

    fn split(&self, s: String) -> Vec<String> {
        let mut cost: Vec<f64> = vec![0.0];

        for i in 1..s.len() + 1 {
            let (_, k) = self.best_match(i, &cost, &s);
            cost.push(k);
        }

        let mut out: Vec<String> = Vec::new();
        let mut i = s.len();
        while i > 0 {
            let (k, c) = self.best_match(i, &cost, &s);
            assert!(c == cost[i]);
            let mut new_token = true;
            let z: usize = 0;
            let outlen = out.len();
            let idx = if (i as i8 - k as i8) < 0 {
                0
            } else {
                i - k as usize
            };
            if &s[idx..i] != "'" {
                if &out.len() > &z {
                    let o = &out[out.len() - 1];
                    if out[out.len() - 1] == "'s"
                        || (s.as_bytes()[i - 1].is_ascii_digit()
                            && o.as_bytes()[0].is_ascii_digit())
                    {
                        out[outlen - 1] =
                            format!("{}{}", &s[i - k as usize..i], &out[out.len() - 1]);
                        new_token = false;
                    }
                }
            }

            if new_token {
                out.push(s[i - k as usize..i].to_string());
            }
            i -= k as usize;
        }
        out.reverse();
        out
    }

    fn best_match(&self, i: usize, cost: &[f64], s: &str) -> (u8, f64) {
        let candidates = &cost[i.saturating_sub(self.max_wlen)..i];
        let mut storedmin: (u8, f64) = (0, 9e99);

        for (index, &candidate) in candidates.into_iter().rev().enumerate() {
            let current_word_cost = match self
                .word_cost
                .get(&s[i - index - 1..i].to_ascii_lowercase())
            {
                Some(value) => value,
                _ => &(9e99 as f64),
            };
            if candidate + current_word_cost < storedmin.1 {
                storedmin.1 = candidate + *current_word_cost;
                storedmin.0 = index as u8 + 1;
            }
        }

        storedmin
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic_split() {
        let no_spaces = "thequickbrownfoxjumpedoverthelazydog".to_string();
        let with_spaces = "the quick brown fox jumped over the lazy dog";
        let lm = LanguageModel::new();
        let correct: Vec<&str> = with_spaces.split_whitespace().collect();

        assert_eq!(lm.split(no_spaces), correct);
    }

    #[test]
    fn split_with_punctuation() {
        let no_spaces = "thequick!brownfox.jumpedoverthe,lazydog?".to_string();
        let with_spaces = "the quick brown fox jumped over the lazy dog";
        let lm = LanguageModel::new();
        let correct: Vec<&str> = with_spaces.split_whitespace().collect();

        assert_eq!(lm.split(no_spaces), correct);
    }
}
