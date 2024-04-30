use anyhow::{bail,Result};
use tz::UtcDateTime;
use regex::Regex;

use crate::{
    boolean::Expr,
    fstok::Token,
    fsexpr::{FsAtom,FsDate}
};

impl Expr<FsAtom> {
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

    fn eat_one(u:&[Token])->Result<(Self,&[Token])> {
	match u {
	    [Token::False,rest @ ..] => Ok((Expr::False,rest)),
	    [Token::True,rest @ ..] => Ok((Expr::True,rest)),
	    [Token::Drive,Token::Unsigned(x),rest @ ..] =>
		Ok((Expr::Atom(FsAtom::Drive(*x)),rest)),
	    [Token::Smaller,Token::Unsigned(x),rest @ ..] =>
		Ok((Expr::Atom(FsAtom::Smaller(*x)),rest)),
	    [Token::Larger,Token::Unsigned(x),rest @ ..] =>
		Ok((Expr::Atom(FsAtom::Larger(*x)),rest)),
	    [Token::Before,Token::Date(d),rest @ ..] =>
		Ok((Expr::Atom(FsAtom::Before(d.to_timestamp()?)),rest)),
	    [Token::After,Token::Date(d),rest @ ..] =>
		Ok((Expr::Atom(FsAtom::After(d.to_timestamp()?)),rest)),
	    [Token::Name,Token::Str(u),rest @ ..] => {
		let rex = Regex::new(u)?;
		Ok((Expr::Atom(FsAtom::NameMatch(rex)),rest))
	    },
	    [Token::Str(u),rest @ ..] => {
		let rex = Regex::new(u)?;
		Ok((Expr::Atom(FsAtom::PathMatch(rex)),rest))
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
