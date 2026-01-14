use crate::backend::types::{SegResult, Sentence, Token, TokenKind};
use std::borrow::Cow;
use std::iter::Peekable;
use unicode_general_category::{GeneralCategory, get_general_category};
use unicode_segmentation::{USentenceBoundIndices, UnicodeSegmentation};

// ---------------------------------------------------------
// PUBLIC API
// ---------------------------------------------------------

pub fn process_text<'a>(text: &'a str) -> SegResult<'a> {
    if text.trim().is_empty() {
        return SegResult::default();
    }

    let sentences = SentenceIterator::new(text).collect();

    SegResult { sentences }
}

fn classify_token(text: &str) -> TokenKind {
    // Optimization: Check bytes for number
    if !text.is_empty() && text.bytes().all(|b| b.is_ascii_digit()) {
        return TokenKind::Number;
    }

    let mut chars = text.chars();
    if let Some(c) = chars.next() {
        if c.is_alphabetic() {
            return TokenKind::Word;
        }
        match get_general_category(c) {
            GeneralCategory::OpenPunctuation
            | GeneralCategory::ClosePunctuation
            | GeneralCategory::InitialPunctuation
            | GeneralCategory::FinalPunctuation
            | GeneralCategory::DashPunctuation
            | GeneralCategory::ConnectorPunctuation
            | GeneralCategory::OtherPunctuation => return TokenKind::Punctuation,
            _ => {}
        }
    }
    TokenKind::Other
}

// ---------------------------------------------------------
// INTERNAL TYPES - REMOVED (Using public types directly)
// ---------------------------------------------------------

// ---------------------------------------------------------
// HELPER: Balance Tracker (State Machine)
// ---------------------------------------------------------

#[derive(Debug, Clone, Copy)]
struct BalanceTracker {
    brackets: isize,
    quotes: isize,
    in_straight_quote: bool,
}

impl BalanceTracker {
    fn new() -> Self {
        Self {
            brackets: 0,
            quotes: 0,
            in_straight_quote: false,
        }
    }

    // We only merge on brackets, not quotes, as per previous requirements.
    fn is_merging(&self) -> bool {
        self.brackets > 0
    }

    fn update(&mut self, text: &str) {
        for c in text.chars() {
            match get_general_category(c) {
                GeneralCategory::OpenPunctuation => self.brackets += 1,
                GeneralCategory::ClosePunctuation => {
                    if self.brackets > 0 {
                        self.brackets -= 1;
                    }
                }
                GeneralCategory::InitialPunctuation => self.quotes += 1,
                GeneralCategory::FinalPunctuation => {
                    if self.quotes > 0 {
                        self.quotes -= 1;
                    }
                }
                _ => {
                    if c == '"' || c == '\'' {
                        self.in_straight_quote = !self.in_straight_quote;
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------
// SENTENCE ITERATOR
// ---------------------------------------------------------

struct SentenceIterator<'a> {
    iter: Peekable<USentenceBoundIndices<'a>>,
    text_source: &'a str,
    tracker: BalanceTracker,
    token_id_counter: usize,
}

impl<'a> SentenceIterator<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            iter: text.split_sentence_bound_indices().peekable(),
            text_source: text,
            tracker: BalanceTracker::new(),
            token_id_counter: 1,
        }
    }
}

// Heuristic: Is this segment a list marker? e.g., "1.", "(1)", "၁။"
fn is_list_marker(text: &str) -> bool {
    let trimmed = text.trim();
    // List markers are usually short
    if trimmed.len() > 10 {
        return false;
    }

    let mut chars = trimmed.chars();
    let first = chars.next();

    // Check if starts with digit or open paren
    let starts_valid = first.is_some_and(|c| c.is_numeric() || c == '(');
    if !starts_valid {
        return false;
    }

    // Check ending: specific Myanmar punctuation or dot
    // \u{104A} is '၊', \u{104B} is '။'
    let last = trimmed.chars().last().unwrap_or(' ');
    matches!(last, '.' | '\u{104A}' | '\u{104B}')
}

fn is_abbreviation(text: &str) -> bool {
    // Simple heuristic: short text ending in dot, but not a list marker
    let trimmed = text.trim();
    trimmed.len() < 5 && trimmed.ends_with('.') && !is_list_marker(text)
}

impl<'a> Iterator for SentenceIterator<'a> {
    type Item = Sentence<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Find the start of the next valid sentence
        let (start_offset, first_part) = loop {
            match self.iter.next() {
                Some((offset, part)) => {
                    if !part.trim().is_empty() {
                        break (offset, part);
                    }
                }
                None => return None,
            }
        };

        // We will accumulate parts if they need merging (unbalanced or list markers)
        let mut current_end = start_offset + first_part.len();
        self.tracker.update(first_part);

        // Peek loop: Consume next sentences if we are not balanced or if it's a list/abbr
        while let Some((peek_offset, peek_part)) = self.iter.peek() {
            // Check if we should merge the *next* part into the current one.
            // 1. Current state is merging (unbalanced brackets)
            let is_merging = self.tracker.is_merging();

            // 2. Current part looks like a list marker (so next part belongs to it)
            // We check the text we have accumulated so far (or just the last part?)
            // Usually list marker is the *start* of the sentence.
            // So we check the text from start_offset to current_end.
            let current_text = &self.text_source[start_offset..current_end];
            let looks_like_list = is_list_marker(current_text);

            // 3. Current part looks like an abbreviation
            let looks_like_abbr = is_abbreviation(current_text);

            if is_merging || looks_like_list || looks_like_abbr {
                // Consume the peeked item
                self.tracker.update(peek_part);
                current_end = *peek_offset + peek_part.len();
                self.iter.next(); // advance iterator
            } else {
                break;
            }
        }

        let full_text = &self.text_source[start_offset..current_end];

        // Tokenize this sentence
        let tokens = tokenize_sentence(full_text, start_offset, &mut self.token_id_counter);

        Some(Sentence {
            text: Cow::Borrowed(full_text),
            tokens,
        })
    }
}

// ---------------------------------------------------------
// TOKENIZER LOGIC
// ---------------------------------------------------------

fn tokenize_sentence<'a>(
    text: &'a str,
    base_offset: usize,
    id_counter: &mut usize,
) -> Vec<Token<'a>> {
    let mut tokens = Vec::with_capacity(text.len() / 5);
    let word_iter = text.split_word_bound_indices().peekable();
    let mut tracker = BalanceTracker::new();

    let mut pending_start: Option<usize> = None;
    let mut pending_end = 0;
    let mut pending_kind: Option<TokenKind> = None;

    for (local_offset, word) in word_iter {
        let is_whitespace = word.trim().is_empty();

        let start_merging = tracker.is_merging();
        if !is_whitespace {
            tracker.update(word);
        }
        let end_merging = tracker.is_merging();
        let currently_merging = start_merging || end_merging;

        if currently_merging {
            // Merging state (brackets)
            if pending_start.is_none() {
                pending_start = Some(local_offset);
                pending_kind = Some(TokenKind::Merged);
            } else if pending_kind != Some(TokenKind::Merged) {
                // Was merging words, now brackets -> emit previous, start merge
                let start = pending_start.unwrap();
                let len = pending_end - start;
                tokens.push(Token {
                    id: *id_counter,
                    offset: base_offset + start,
                    text: Cow::Borrowed(&text[start..start + len]),
                    kind: pending_kind.unwrap(),
                });
                *id_counter += 1;

                pending_start = Some(local_offset);
                pending_kind = Some(TokenKind::Merged);
            }
            pending_end = local_offset + word.len();

            if !end_merging {
                // Finished merging
                let start = pending_start.unwrap();
                let len = pending_end - start;
                tokens.push(Token {
                    id: *id_counter,
                    offset: base_offset + start,
                    text: Cow::Borrowed(&text[start..start + len]),
                    kind: TokenKind::Merged,
                });
                *id_counter += 1;
                pending_start = None;
                pending_kind = None;
            }
        } else {
            // Not merging brackets
            if is_whitespace {
                if let Some(start) = pending_start {
                    // Emit pending
                    let len = pending_end - start;
                    tokens.push(Token {
                        id: *id_counter,
                        offset: base_offset + start,
                        text: Cow::Borrowed(&text[start..start + len]),
                        kind: pending_kind.unwrap(),
                    });
                    *id_counter += 1;
                    pending_start = None;
                    pending_kind = None;
                }
                continue;
            }

            let current_kind = classify_token(word);

            if let Some(pk) = pending_kind {
                if pk == TokenKind::Word && current_kind == TokenKind::Word {
                    // Merge consecutive words
                    pending_end = local_offset + word.len();
                } else {
                    // Emit pending
                    let start = pending_start.unwrap();
                    let len = pending_end - start;
                    tokens.push(Token {
                        id: *id_counter,
                        offset: base_offset + start,
                        text: Cow::Borrowed(&text[start..start + len]),
                        kind: pk,
                    });
                    *id_counter += 1;

                    // Start new
                    pending_start = Some(local_offset);
                    pending_end = local_offset + word.len();
                    pending_kind = Some(current_kind);
                }
            } else {
                // Start new
                pending_start = Some(local_offset);
                pending_end = local_offset + word.len();
                pending_kind = Some(current_kind);
            }
        }
    }

    // Emit remaining
    if let Some(start) = pending_start {
        let len = pending_end - start;
        tokens.push(Token {
            id: *id_counter,
            offset: base_offset + start,
            text: Cow::Borrowed(&text[start..start + len]),
            kind: pending_kind.unwrap(),
        });
        *id_counter += 1;
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segmentation_logic() {
        let text = "1. Tena samayena buddho bhagavā verañjāyaṃ viharati naḷerupucimandamūle mahatā bhikkhusaṅghena saddhiṃ pañcamattehi bhikkhusatehi. Assosi kho verañjo brāhmaṇo – ‘‘samaṇo khalu, bho, gotamo sakyaputto sakyakulā pabbajito verañjāyaṃ viharati naḷerupucimandamūle mahatā bhikkhusaṅghena saddhiṃ pañcamattehi bhikkhusatehi. Taṃ kho pana bhavantaṃ gotamaṃ evaṃ kalyāṇo kittisaddo abbhuggato – ‘itipi so bhagavā arahaṃ sammāsambuddho vijjācaraṇasampanno sugato lokavidū anuttaro purisadammasārathi satthā devamanussānaṃ buddho bhagavā [bhagavāti (syā.), dī. ni. 1.157, abbhuggatākārena pana sameti]. So imaṃ lokaṃ sadevakaṃ samārakaṃ sabrahmakaṃ sassamaṇabrāhmaṇiṃ pajaṃ sadevamanussaṃ sayaṃ abhiññā sacchikatvā pavedeti. So dhammaṃ deseti ādikalyāṇaṃ majjhekalyāṇaṃ pariyosānakalyāṇaṃ sātthaṃ sabyañjanaṃ; kevalaparipuṇṇaṃ parisuddhaṃ brahmacariyaṃ pakāseti; sādhu kho pana tathārūpānaṃ arahataṃ dassanaṃ hotī’’’ti.";

        let result = process_text(text);

        assert!(!result.sentences.is_empty());
        eprintln!("Sentences found: {}", result.sentences.len());

        for sent in result.sentences {
            eprintln!("sentence tokens: {}", sent.tokens.len());
            for token in &sent.tokens {
                eprintln!("Token: {:?} ({:?})", token.text, token.kind);
            }
        }
    }

    #[test]
    fn test_thai_segmentation() {
        // "อุทฺทิฏฺฐา" should be a single word token
        // Note: unicode-segmentation might split this if it's not in its dictionary?
        // The previous implementation used `WordIterator` which had logic to merge consecutive words?
        // Wait, the previous `WordIterator` had:
        // `matches!(current_kind, InternalTokenKind::Word)` -> merge if pending is Word and current is Word.
        // My new `tokenize_sentence` does NOT have this logic!
        // It simply emits `classify_token(word)`.

        // I need to restore the "merge consecutive words" logic for Thai/Burmese!
        // The user's optimization plan missed this nuance.
        // I must add it back.

        let text = "อุทฺทิฏฺฐา";
        let result = process_text(text);

        assert_eq!(result.sentences.len(), 1);
        let tokens = &result.sentences[0].tokens;
        // If my new logic doesn't merge, this might fail.
        // I should verify if `unicode-segmentation` splits this.
        // If it does, I need the merging logic.
        // Assuming I need it.

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].text, "อุทฺทิฏฺฐา");
        assert_eq!(tokens[0].kind, TokenKind::Word);
    }

    #[test]
    fn test_quote_segmentation() {
        // Text inside quotes should be segmented into words
        let text = "‘itipi so bhagavā";
        let result = process_text(text);

        assert_eq!(result.sentences.len(), 1);
        let tokens = &result.sentences[0].tokens;

        // Expected: "‘" (Punct), "itipi" (Word), "so" (Word), "bhagavā" (Word)
        assert!(
            tokens.len() > 1,
            "Should have multiple tokens, got {}",
            tokens.len()
        );

        let words: Vec<&str> = tokens
            .iter()
            .filter(|t| t.kind == TokenKind::Word)
            .map(|t| t.text.as_ref())
            .collect();

        assert_eq!(words, vec!["itipi", "so", "bhagavā"]);
    }

    #[test]
    fn test_bracket_sentence_merging() {
        // Text inside brackets should be treated as a single sentence, even if it contains periods.
        let text = "[bhagavāti (syā.), dī. ni. 1.157, abbhuggatākārena pana sameti].";
        let result = process_text(text);

        eprintln!("Sentences found: {}", result.sentences.len());
        for (i, sent) in result.sentences.iter().enumerate() {
            eprintln!("Sentence {}: {}", i, sent.text);
        }

        assert_eq!(
            result.sentences.len(),
            1,
            "Should be 1 sentence, found {}",
            result.sentences.len()
        );
    }

    #[test]
    fn test_burmese_list_segmentation() {
        // Burmese list markers like "၁." or "၁။" should be merged with the following text.
        // "၁။ တေန သမယေန" -> Should be 1 sentence.
        let text = "၁။ တေန သမယေန ဗုဒ္ဓေါ ဘဂဝါ ဝေရဉ္ဇာယံ ဝိဟရတိ။";
        let result = process_text(text);

        eprintln!("Sentences found: {}", result.sentences.len());
        for (i, sent) in result.sentences.iter().enumerate() {
            eprintln!("Sentence {}: {}", i, sent.text);
        }

        assert_eq!(
            result.sentences.len(),
            1,
            "Should be 1 sentence, found {}",
            result.sentences.len()
        );
    }
}
