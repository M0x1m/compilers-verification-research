use crate::*;
use crate::lexer::*;

#[derive(Debug)]
pub struct AST {
    vars: HashMap<String, String>,
    funcs: HashMap<String, Func>,
}

#[derive(Debug)]
enum ParseError<'a> {
    LexerError(LexerError<'a>),
    UnexpectedToken(TokenWithLoc<'a>),
    Unimplemented(TokenWithLoc<'a>),
}

fn parser_expect_token<'a>(exp: Token<'_>, l: &mut Lexer<'a>) -> Result<TokenWithLoc<'a>, ParseError<'a>> {
    let act = l.next_token()?;
    if act.tok == exp {
        Ok(act)
    } else {
        Err(ParseError::UnexpectedToken(act))
    }
}

fn parser_expect_token_id<'a>(l: &mut Lexer<'a>) -> Result<&'a str, ParseError<'a>> {
    match l.next_token()? {
        TokenWithLoc{tok: Token::Id(id), ..} => Ok(id),
        other => Err(ParseError::UnexpectedToken(other))
    }
}

fn parse_vars_decl<'a>(vars: &mut HashMap<String, String>, l: &mut Lexer<'a>) -> Result<(), ParseError<'a>> {
    let mut names = vec![];

    loop {
        match l.next_token()? {
            TokenWithLoc{tok: Token::Id(name), ..} => {
                names.push(name.to_string());
            }
            other => return Err(ParseError::UnexpectedToken(other))
        }
        if l.peek_token()?.tok == Token::Colon { break }
        parser_expect_token(Token::Comma, l)?;
    }

    parser_expect_token(Token::Colon, l)?;
    let typ = parser_expect_token_id(l)?;

    for name in names {
        vars.insert(name, typ.to_string());
    }

    Ok(())
}

#[derive(Clone, Debug)]
enum Expr {
    Name(String),
    Number(i32),
    FunCall(Box<Expr>, Vec<Expr>),
    Assign(Box<Expr>, Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
}

fn parse_expr_primary<'a>(l: &mut Lexer<'a>) -> Result<Expr, ParseError<'a>> {
    match l.peek_token()? {
        TokenWithLoc{tok: Token::OpenParen, ..} => {
            parser_expect_token(Token::OpenParen, l)?;
            let ret = parse_expr(l)?;
            parser_expect_token(Token::CloseParen, l)?;
            Ok(ret)
        }
        TokenWithLoc{tok: Token::Id(id), ..} => {
            let _ = l.next_token()?;
            Ok(Expr::Name(id.to_string()))
        }
        TokenWithLoc{tok: Token::Number(n), ..} => {
            let _ = l.next_token()?;
            Ok(Expr::Number(n))
        }
        t => Err(ParseError::Unimplemented(t))
    }
}

fn parse_expr<'a>(l: &mut Lexer<'a>) -> Result<Expr, ParseError<'a>> {
    let mut lhs = parse_expr_primary(l)?;
    loop {
        match l.peek_token()? {
            TokenWithLoc{tok: Token::OpenParen, ..} => {
                let mut args = vec![];
                parser_expect_token(Token::OpenParen, l)?;
                while l.peek_token()?.tok != Token::CloseParen {
                    let expr = parse_expr(l)?;
                    args.push(expr);
                    if l.peek_token()?.tok == Token::CloseParen { break }
                    parser_expect_token(Token::Comma, l)?;
                }

                parser_expect_token(Token::CloseParen, l)?;
                lhs = Expr::FunCall(lhs.into(), args)
            },
            TokenWithLoc{tok: Token::Plus, ..} => {
                parser_expect_token(Token::Plus, l)?;
                lhs = Expr::Add(lhs.into(), parse_expr_primary(l)?.into())
            }
            TokenWithLoc{tok: Token::Eq, ..} => {
                parser_expect_token(Token::Eq, l)?;
                lhs = Expr::Assign(lhs.into(), parse_expr(l)?.into())
            }
            TokenWithLoc{tok: Token::Id(_), ..} |
            TokenWithLoc{tok: Token::CloseCurly, ..} |
            TokenWithLoc{tok: Token::CloseParen, ..} => break Ok(lhs),
            t => break Err(ParseError::UnexpectedToken(t))
        }
    }
}

fn parse_func_body<'a>(l: &mut Lexer<'a>) -> Result<Func, ParseError<'a>> {
    let mut func = Func { exprs: vec![], vars: HashMap::new() };

    while l.peek_token()?.tok != Token::CloseCurly {
        let saved_l = l.clone();
        match parse_expr(l) {
            Ok(expr) => func.exprs.push((saved_l.get_loc(), expr)),
            Err(err) => {
                *l = saved_l;
                parse_vars_decl(&mut func.vars, l).map_err(|_| err)?
            }
        }
    }

    Ok(func)
}

fn parse_func<'a>(f: &mut AST, l: &mut Lexer<'a>) -> Result<(), ParseError<'a>> {
    parser_expect_token(Token::Fn, l)?;
    let func_name = parser_expect_token_id(l)?;
    parser_expect_token(Token::OpenParen, l)?;
    parser_expect_token(Token::CloseParen, l)?;
    parser_expect_token(Token::OpenCurly, l)?;

    let exprs = parse_func_body(l)?;

    parser_expect_token(Token::CloseCurly, l)?;

    f.funcs.insert(func_name.to_string(), exprs);
    Ok(())
}

fn parse_top_level<'a>(l: &mut Lexer<'a>) -> Result<AST, ParseError<'a>> {
    let mut f = AST { vars: HashMap::new(), funcs: HashMap::new() };

    loop {
        match l.peek_token()? {
            TokenWithLoc{tok: Token::Fn, ..} => parse_func(&mut f, l)?,
            TokenWithLoc{tok: Token::Id(_), ..} => parse_vars_decl(&mut f.vars, l)?,
            TokenWithLoc{tok: Token::Eof, ..} => break,
            tok => todo!("{tok:?}")
        }
    }

    Ok(f)
}

pub fn parse_file(filename: &str) -> Result<AST, Box<dyn std::error::Error>> {
    let mut content = String::new();
    fs::File::open(filename)?.read_to_string(&mut content)?;

    let mut l = Lexer::new(&content, filename);
    Ok(parse_top_level(&mut l).map_err(|err| err.to_string())?)
}

#[derive(Debug)]
struct Func {
    vars: HashMap<String, String>,
    exprs: Vec<(LexerLocRc, Expr)>,
}

impl std::error::Error for ParseError<'_> {}

impl std::fmt::Display for ParseError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::LexerError(le) => le.fmt(f)?,
            Self::UnexpectedToken(tok) => write!(f, "{loc}: error: unexpected token {tok}", loc = tok.loc, tok = tok.tok)?,
            Self::Unimplemented(tok) => write!(f, "{loc}: error: unimplemented {tok}", loc = tok.loc, tok = tok.tok)?,
        }
        Ok(())
    }
}

impl<'a> From<LexerError<'a>> for ParseError<'a> {
    fn from(le: LexerError<'a>) -> ParseError<'a> {
        ParseError::LexerError(le)
    }
}
