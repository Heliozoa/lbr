//! Contains SentenceSplitter, an Iterator that iterates through Japanese sentences in a string.

#[derive(Debug, Clone)]
pub struct SentenceSplitter<'a> {
    idx: usize,
    s: &'a str,
    fully_quoted_sentence_ender: Option<char>,
}

impl<'a> SentenceSplitter<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            idx: 0,
            s: s.trim(),
            fully_quoted_sentence_ender: None,
        }
    }
}

impl<'a> Iterator for SentenceSplitter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        // scroll past whitespace
        for c in self.s[self.idx..].chars() {
            if c.is_whitespace() {
                self.idx += c.len_utf8();
            } else {
                break;
            }
        }

        // check quote sentence
        if let Some(ender) = self.fully_quoted_sentence_ender {
            let first = self.s[self.idx..].chars().next()?;
            if first == ender {
                // skip over ender
                self.idx += first.len_utf8();
                self.fully_quoted_sentence_ender =
                    fully_quoted_sentence_ender(self.s.get(self.idx..)?);
                if let Some(ender) = self.fully_quoted_sentence_ender {
                    self.idx += ender.len_utf8();
                }
            }
        } else {
            self.fully_quoted_sentence_ender = fully_quoted_sentence_ender(self.s.get(self.idx..)?);
            if let Some(ender) = self.fully_quoted_sentence_ender {
                self.idx += ender.len_utf8();
            }
        }

        let start_idx = self.idx;
        let next_chunk = self.s.get(start_idx..)?;
        if next_chunk.is_empty() {
            return None;
        }

        let mut at_sentence_end = false;
        let mut quote_stack = Vec::<char>::new();
        let sentence_enders = "。？！…‥.?!";

        // go through each character
        for c in next_chunk.chars() {
            // check for end-of-sentence and end-of-end-of-sentence if not in quote
            if sentence_enders.contains(c) && quote_stack.is_empty() {
                at_sentence_end = true;
            } else if at_sentence_end && quote_stack.is_empty() {
                // sentence over
                return Some(self.s[start_idx..self.idx].trim_start());
            }
            // check for end of fully quoted sentence
            if quote_stack.is_empty()
                && self
                    .fully_quoted_sentence_ender
                    .map(|e| e == c)
                    .unwrap_or_default()
            {
                // end of sentence quote, skip over last quote
                let end_idx = self.idx;
                self.idx += c.len_utf8();
                self.fully_quoted_sentence_ender = None;
                return Some(self.s[start_idx..end_idx].trim_start());
            }
            self.idx += c.len_utf8();

            // check for end of quote
            if quote_stack.last().map(|f| f == &c).unwrap_or_default() {
                quote_stack.pop();
            }
            // check for start of quote
            if let Some(quote) = corresponding_quote(c) {
                quote_stack.push(quote)
            }
        }

        Some(self.s[start_idx..].trim())
    }
}

fn corresponding_quote(c: char) -> Option<char> {
    let q = match c {
        '｛' => '｝',
        '（' => '）',
        '［' => '］',
        '【' => '】',
        '「' => '」',
        '『' => '』',
        '〝' => '〟',
        _ => return None,
    };
    Some(q)
}

fn fully_quoted_sentence_ender(s: &str) -> Option<char> {
    let ending_quote = s.chars().next().and_then(corresponding_quote)?;
    let mut processing_idx = ending_quote.len_utf8();

    let mut quote_stack = vec![ending_quote];
    for c in s.get(processing_idx..)?.chars() {
        processing_idx += c.len_utf8();
        if let Some(closing) = corresponding_quote(c) {
            quote_stack.push(closing);
        }
        if quote_stack.last().map(|f| f == &c).unwrap_or_default() {
            quote_stack.pop();
        }
        if quote_stack.is_empty() {
            if let Some(next_char) = s.get(processing_idx..).and_then(|s| s.chars().next()) {
                if next_char.is_whitespace() || corresponding_quote(next_char).is_some() {
                    return Some(ending_quote);
                } else {
                    return None;
                }
            } else {
                // no next char, fully quoted
                return Some(ending_quote);
            }
        }
    }
    // mismatched quotes
    None
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn single_sentence() {
        let sentence = "おはよう。";
        let ss = SentenceSplitter::new(sentence);
        let sentences = ss.collect::<Vec<_>>();
        assert_eq!(sentences, &[sentence]);
    }

    #[test]
    fn two_sentences() {
        let sentences = "おはよう。さようなら。";
        let ss = SentenceSplitter::new(&sentences);
        let sentences = ss.collect::<Vec<_>>();
        assert_eq!(sentences, &["おはよう。", "さようなら。"]);
    }

    #[test]
    fn quotes_as_part_of_sentence() {
        let sentence = "「おはよう。」と「さようなら。」と言った。";
        let ss = SentenceSplitter::new(sentence);
        let sentences = ss.collect::<Vec<_>>();
        assert_eq!(sentences, &["「おはよう。」と「さようなら。」と言った。"]);
    }

    #[test]
    fn fully_quoted_sentence() {
        let sentence = "「おはよう。」";
        let ss = SentenceSplitter::new(sentence);
        let sentences = ss.collect::<Vec<_>>();
        assert_eq!(sentences, &["おはよう。"]);
    }

    #[test]
    fn doubly_quoted() {
        let sentences = "「『おはよう。』と『さようなら。』と言った。」";
        let ss = SentenceSplitter::new(sentences);
        let sentences = ss.collect::<Vec<_>>();
        assert_eq!(sentences, &["『おはよう。』と『さようなら。』と言った。"]);
    }

    #[test]
    fn consecutive_quotes() {
        let sentences = "「おはよう。」「さようなら。」";
        let ss = SentenceSplitter::new(sentences);
        let sentences = ss.collect::<Vec<_>>();
        assert_eq!(sentences, &["おはよう。", "さようなら。"]);
    }

    #[test]
    fn fully_quoted() {
        assert_eq!('」', fully_quoted_sentence_ender("「なるほど。」").unwrap());
    }

    #[test]
    fn quoted_in_sentence() {
        assert!(fully_quoted_sentence_ender("「なるほど。」と言った。").is_none());
    }

    #[test]
    fn consecutive_quotes_sentence_ender() {
        assert_eq!(
            '」',
            fully_quoted_sentence_ender("「なるほど。」『なるほど。』").unwrap()
        );
    }

    #[test]
    fn multiple_sentences_in_quote_sentence() {
        let sentences = "「おはよう。さようなら。」";
        let ss = SentenceSplitter::new(sentences);
        let sentences = ss.collect::<Vec<_>>();
        assert_eq!(sentences, &["おはよう。", "さようなら。"]);
    }

    #[test]
    fn dots_followed_by_interrobang() {
        let sentences = "「おはよう...!? さようなら。」";
        let ss = SentenceSplitter::new(sentences);
        let sentences = ss.collect::<Vec<_>>();
        assert_eq!(sentences, &["おはよう...!?", "さようなら。"]);
    }
}
