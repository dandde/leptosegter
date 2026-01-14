use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SegResult<'a> {
    pub sentences: Vec<Sentence<'a>>,
}

impl<'a> SegResult<'a> {
    pub fn to_owned_data(&self) -> SegResult<'static> {
        SegResult {
            sentences: self.sentences.iter().map(|s| s.to_owned_data()).collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Sentence<'a> {
    pub text: Cow<'a, str>,
    pub tokens: Vec<Token<'a>>,
}

impl<'a> Sentence<'a> {
    pub fn to_owned_data(&self) -> Sentence<'static> {
        Sentence {
            text: Cow::Owned(self.text.clone().into_owned()),
            tokens: self.tokens.iter().map(|t| t.to_owned_data()).collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token<'a> {
    pub id: usize,
    pub offset: usize,
    pub text: Cow<'a, str>,
    pub kind: TokenKind,
}

impl<'a> Token<'a> {
    pub fn to_owned_data(&self) -> Token<'static> {
        Token {
            id: self.id,
            offset: self.offset,
            text: Cow::Owned(self.text.clone().into_owned()),
            kind: self.kind,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum TokenKind {
    Word,
    Number,
    Punctuation,
    Merged,
    Other,
}
