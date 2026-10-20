use Rc;

#[derive(PartialEq, Debug)]
pub enum Token<'a> {
    Eof,
    Id(&'a str),
    Number(i32),
    Fn,
    OpenParen,
    CloseParen,
    OpenCurly,
    CloseCurly,
    Comma,
    Colon,
    Eq,
    Plus,
}

#[derive(Debug)]
pub struct TokenWithLoc<'a> {
    pub tok: Token<'a>,
    pub loc: LexerLoc<'a>,
}

#[derive(Clone, Copy, Debug)]
pub struct LexerLoc<'a> {
    filename: &'a str,
    row: u32,
    col: u32,
}

#[derive(Clone, Debug)]
pub struct LexerLocRc {
    filename: Rc<str>,
    row: u32,
    col: u32,
}

impl std::fmt::Display for LexerLoc<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}:{}:{}", self.filename, self.row + 1, self.col + 1)?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct Lexer<'a> {
    source: &'a str,
    loc: LexerLoc<'a>,
}

#[derive(Debug)]
pub enum LexerError<'a> {
    UnknownToken(LexerLoc<'a>),
    IntParseError(LexerLoc<'a>),
}

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Eof => write!(f, "end of file")?,
            Self::Id(_) => write!(f, "identifier")?,
            Self::Number(_) => write!(f, "number")?,
            Self::OpenParen => write!(f, "'('")?,
            Self::CloseParen => write!(f, "')'")?,
            Self::OpenCurly => write!(f, "'{{'")?,
            Self::CloseCurly => write!(f, "'}}'")?,
            Self::Comma => write!(f, "','")?,
            Self::Colon => write!(f, "':'")?,
            Self::Fn => write!(f, "'fn'")?,
            Self::Eq => write!(f, "'='")?,
            Self::Plus => write!(f, "'+'")?,
        }
        Ok(())
    }
}

impl std::error::Error for LexerError<'_> {}

impl std::fmt::Display for LexerError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::UnknownToken(loc) => write!(f, "{loc}: error: unknown token")?,
            Self::IntParseError(loc) => write!(f, "{loc}: error: invalid integer literal")?,
        }
        Ok(())
    }
}

const SINGLE_CHAR_TOKENS: [(char, Token<'_>); 8] = [
    ('(', Token::OpenParen),
    (')', Token::CloseParen),
    ('{', Token::OpenCurly),
    ('}', Token::CloseCurly),
    (',', Token::Comma),
    (':', Token::Colon),
    ('=', Token::Eq),
    ('+', Token::Plus),
];

const KEYWORDS_TOKENS: [(&str, Token<'_>); 1] = [
    ("fn", Token::Fn)
];

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str, filename: &'a str) -> Lexer<'a> {
        Self {
            source,
            loc: LexerLoc {
                filename,
                row: 0,
                col: 0
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        let mut chars = self.source.chars();
        chars.next()
    }

    fn get_char(&mut self) -> Option<char> {
        self.peek_char().map(|ch| {
            match ch {
                '\n' => {
                    self.loc.row += 1;
                    self.loc.col = 0;
                }
                _ => {
                    self.loc.col += 1;
                }
            }

            self.source = &self.source[ch.len_utf8()..];
            ch
        })
    }

    pub fn get_loc(&self) -> LexerLocRc {
        LexerLocRc {
            filename: self.loc.filename.into(),
            row: self.loc.row,
            col: self.loc.col,
        }
    }

    pub fn peek_token(&self) -> Result<TokenWithLoc<'a>, LexerError<'a>> {
        let mut tok = self.clone();
        tok.next_token()
    }

    pub fn next_token(&mut self) -> Result<TokenWithLoc<'a>, LexerError<'a>> {
        while self.peek_char().is_some_and(char::is_whitespace) {
            let _ = self.get_char();
        }

        if self.peek_char().is_none() {
            return Ok(TokenWithLoc {tok: Token::Eof, loc: self.loc})
        }

        'kwloop: for (kw, tok) in KEYWORDS_TOKENS {
            let loc = self.loc;
            let saved = self.clone();
            for ch in kw.chars() {
                if self.peek_char() != Some(ch) {
                    *self = saved;
                    continue 'kwloop
                }
                let _ = self.get_char();
            }
            if self.peek_char().is_some_and(char::is_alphanumeric) {
                *self = saved;
                continue
            }
            return Ok(TokenWithLoc {tok, loc})
        }

        for (ch, tok) in SINGLE_CHAR_TOKENS {
            if self.peek_char() != Some(ch) { continue }
            let loc = self.loc;
            let _ = self.get_char();
            return Ok(TokenWithLoc {tok, loc})
        }

        if self.peek_char().is_some_and(char::is_alphabetic) {
            let tk_loc = self.loc;
            let tk_id = self.source;
            let mut tk_len = 0;
            while self.peek_char().is_some_and(|x| x.is_alphanumeric() || x == '_') {
                tk_len += self.get_char().unwrap().len_utf8();
            }
            return Ok(TokenWithLoc {
                tok: Token::Id(&tk_id[..tk_len]),
                loc: tk_loc
            });
        }

        if self.peek_char().is_some_and(char::is_numeric) {
            let tk_loc = self.loc;
            let tk_id = self.source;
            let mut tk_len = 0;
            while self.peek_char().is_some_and(char::is_numeric) {
                tk_len += self.get_char().unwrap().len_utf8();
            }
            return Ok(TokenWithLoc {
                tok: Token::Number(tk_id[..tk_len].parse().map_err(|_| LexerError::IntParseError(tk_loc))?),
                loc: tk_loc
            });
        }
        Err(LexerError::UnknownToken(self.loc))
    }
}
