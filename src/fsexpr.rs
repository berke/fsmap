use anyhow::{bail,Result};
use regex::Regex;

#[derive(Debug,Clone)]
pub enum Expr<T> {
    True,
    False,
    Atom(T),
    And(Box<Expr<T>>,Box<Expr<T>>),
    Or(Box<Expr<T>>,Box<Expr<T>>),
    Diff(Box<Expr<T>>,Box<Expr<T>>),
}

#[derive(Debug,Clone)]
pub enum Token {
    False,
    True,
    Str(String),
    And,
    Or,
    Diff,
    LPar,
    RPar,
    Eof
}

pub trait Predicate {
    fn test(&self,path:&str)->bool;
}

impl<T> Expr<T> {
    fn eval<F:Fn(&T)->bool>(&self,f:&F)->bool {
	match self {
	    Self::True => true,
	    Self::False => false,
	    Self::Atom(a) => f(a),
	    Self::And(x,y) => x.eval(f) && y.eval(f),
	    Self::Or(x,y) => x.eval(f) || y.eval(f),
	    Self::Diff(x,y) => x.eval(f) && !y.eval(f)
	}
    }
}

impl Predicate for Expr<Regex> {
    fn test(&self,u:&str)->bool {
	self.eval(&|re:&Regex| re.is_match(u))
    }
}

impl Expr<Regex> {
    pub fn parse(u:&str)->Result<Self> {
	let toks = Token::tokenize(u)?;
	Self::parse_from_tokens(&toks[..])
    }

    fn parse_from_tokens(u:&[Token])->Result<Self> {
	let (x,rest) = Self::eat(u)?;
	match rest {
	    [] => Ok(x),
	    _ => bail!("Junk at end of expression")
	}
    }
    
    fn eat(u:&[Token])->Result<(Self,&[Token])> {
	let (x,rest) = Self::eat_one(u)?;
	Self::eat_rest(rest,x)
    }

    fn eat_rest(u:&[Token],x:Self)->Result<(Self,&[Token])> {
	match u {
	    [Token::Diff,rest @ ..] => {
		let (y,rest) = Self::eat_one(rest)?;
		Self::eat_rest(rest,Expr::Diff(Box::new(x),Box::new(y)))
	    },
	    [Token::And,rest @ ..] => Self::eat_and(rest,x),
	    [Token::Or,rest @ ..] => {
		let (y,rest) = Self::eat(rest)?;
		Ok((Expr::Or(Box::new(x),Box::new(y)),rest))
	    },
	    _ => Ok((x,u)),
	}
    }

    fn eat_and(u:&[Token],x:Self)->Result<(Self,&[Token])> {
	let (y,rest) = Self::eat_one(u)?;
	match rest {
	    [Token::And,rest @ ..] => {
		let e = Expr::And(Box::new(x),Box::new(y));
		Self::eat_and(rest,e)}
	    ,
	    [Token::Diff,rest @ ..] => {
		let (z,rest) = Self::eat_one(rest)?;
		Ok((Expr::And(
		    Box::new(x),
		    Box::new(
			Expr::Diff(Box::new(y),Box::new(z))
		    )),
		    rest))
	    },
	    [Token::Or,rest @ ..] => {
		let e = Expr::And(Box::new(x),Box::new(y));
		let (f,rest) = Self::eat(rest)?;
		Ok((Expr::Or(Box::new(e),Box::new(f)),rest))
	    },
	    _ => {
		let e = Expr::And(Box::new(x),Box::new(y));
		Ok((e,rest))
	    }
	}
    }

    fn eat_one(u:&[Token])->Result<(Self,&[Token])> {
	match u {
	    [Token::False,rest @ ..] => Ok((Expr::False,rest)),
	    [Token::True,rest @ ..] => Ok((Expr::True,rest)),
	    [Token::Str(u),rest @ ..] => {
		let rex = Regex::new(u)?;
		Ok((Expr::Atom(rex),rest))
	    },
	    [Token::LPar,rest @ ..] => {
		let (x,rest) = Self::eat(rest)?;
		match rest {
		    [Token::RPar,rest @ ..] => Ok((x,rest)),
		    _ => bail!("Expecting right parenthesis")
		}
	    },
	    _ => bail!("Invalid syntax")
	}
    }
}

impl Token {
    fn eat(u:&[char])->Result<(Self,&[char])> {
	match u {
	    [w,rest @ ..] if w.is_whitespace() => Self::eat(rest),
	    ['%','f',rest @ ..] => Ok((Self::False,rest)),
	    ['%','t',rest @ ..] => Ok((Self::True,rest)),
	    ['&',rest @ ..] => Ok((Self::And,rest)),
	    ['|',rest @ ..] => Ok((Self::Or,rest)),
	    ['\\',rest @ ..] => Ok((Self::Diff,rest)),
	    ['(',rest @ ..] => Ok((Self::LPar,rest)),
	    [')',rest @ ..] => Ok((Self::RPar,rest)),
	    ['\'',rest @ ..] => Self::eat_quoted_str(rest,String::new()),
	    [c @ ('a'..='z'|'A'..='Z'|'0'..='9'
		  |'/'|'.'|','|'*'|'?'|'$'|'^'|'-'|'_'),
	     rest @ ..] => Self::eat_basic_str(rest,(*c).into()),
	    [c,..] => bail!("Unexpected character {:?}",c),
	    [] => Ok((Self::Eof,u))
	}
    }

    fn eat_basic_str(mut u:&[char],mut buf:String)->Result<(Self,&[char])> {
	loop {
	    match u {
		[c @ ('a'..='z'|'A'..='Z'|'0'..='9'|'/'|'.'|','|'*'|'?'|'$'|'^'|'-'|'_'),
		 rest @ ..] => {
		    buf.push(*c);
		    u = rest;
		},
		_ => return Ok((Self::Str(buf),u))
	    }
	}
    }

    fn eat_quoted_str(mut u:&[char],mut buf:String)->Result<(Self,&[char])> {
	loop {
	    match u {
		['\'',rest @ ..] => return Ok((Self::Str(buf),rest)),
		['\\','\\',rest @ ..] => {
		    buf.push('\\');
		    u = rest;
		},
		['\\','\'',rest @ ..] => {
		    buf.push('\'');
		    u = rest;
		},
		[c,rest @ ..] => {
		    buf.push(*c);
		    u = rest;
		},
		[] => bail!("Unexpected EOF in string")
	    }
	}
    }

    fn tokenize(u:&str)->Result<Vec<Self>> {
	let chars : Vec<char> = u.chars().collect();
	let mut res = Vec::new();
	let mut u = &chars[..];
	loop {
	    let (token,rest) = Self::eat(&u)?;
	    if let Token::Eof = token {
		break;
	    }
	    res.push(token);
	    u = rest;
	}
	Ok(res)
    }
}

#[test]
fn test_tokenize() {
    for u in &[
	"'/etc/passwd'",
	"F | FF 'this is an escaped \\'quote\\''    blah blah    ",
	"'foo' | 'bar'",
	"/dead/beef/bad/cafe",
	"/etc/foo.*bar$",
	"a \\\\ backslash",
	"(aaab |b ) &( c |dddd ) \\ (e | f ( g hh)"
	
    ] {
	let toks = Token::tokenize(u).unwrap();
	println!("{:?} -> {:?}",u,toks);
    }
}

#[test]
fn test_parse() {
    for u in &[
	"a",
	"a | b",
	"a | b | c",
	"a",
	"a & b",
	"a & b & c",
	"a | b & c",
	"a & b | c",
	"a | b | c | d",
	"a | b | c & d",
	"a | b & c | d",
	"a | b & c & d",
	"a & b | c | d",
	"a & b | c & d",
	"a & b & c | d",
	"a & b & c & d",
	"a | (b | c)",
	"a | (b | c)",
	"a \\ b",
	"a & b \\ c",
	"a \\ b & c",
    ] {
	let toks = Token::tokenize(u).unwrap();
	let expr = Expr::parse_from_tokens(&toks[..]).unwrap();
	println!("{:?} -> {:?} -> {:?}",u,toks,expr);
    }
}
