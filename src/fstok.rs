use anyhow::{bail,Result};
use tz::UtcDateTime;
use regex::Regex;

use crate::{
    boolean::Expr,
    fsexpr::{FsAtom,FsDate}
};

#[derive(Debug,Clone)]
pub enum Token {
    False,
    True,
    Unsigned(u64),
    Date(FsDate),
    Str(String),
    Drive,
    Name,
    Before,
    After,
    Smaller,
    Larger,
    And,
    Or,
    Diff,
    LPar,
    RPar,
    Eof
}

impl Token {
    fn eat(u:&[char])->Result<(Self,&[char])> {
	match u {
	    [w,rest @ ..] if w.is_whitespace() => Self::eat(rest),
	    ['%',rest @ ..] => {
		let (kw,rest) = Self::eat_keyword(rest,String::new())?;
		let kw =
		    match kw.as_str() {
			"after" => Self::After,
			"before" => Self::Before,
			"drive" => Self::Drive,
			"f" => Self::False,
			"larger" => Self::Larger,
			"name" => Self::Name,
			"smaller" => Self::Smaller,
			"t" => Self::True,
			_ => bail!("Unknown keyword {:?}",kw)
		    };
		Ok((kw,rest))
	    },
	    ['&',rest @ ..] => Ok((Self::And,rest)),
	    ['|',rest @ ..] => Ok((Self::Or,rest)),
	    ['\\',rest @ ..] => Ok((Self::Diff,rest)),
	    ['(',rest @ ..] => Ok((Self::LPar,rest)),
	    [')',rest @ ..] => Ok((Self::RPar,rest)),
	    ['\'',rest @ ..] => Self::eat_quoted_str(rest,String::new()),
	    ['0'..='9',
	     '0'..='9',
	     '0'..='9',
	     '0'..='9',
	     '-',
	     '0'..='9',
	     '0'..='9',
	     '-',
	     '0'..='9',
	     '0'..='9',rest @ ..] => {
		let year = Self::parse_i32(&u[0..4])?;
		let month = Self::parse_i32(&u[5..7])?;
		let day = Self::parse_i32(&u[8..10])?;
		Ok((Self::Date(FsDate { year,month,day }),rest))
	    },
	    ['0'..='9',..] => {
		let (n,rest) = Self::parse_size(u)?;
		Ok((Self::Unsigned(n),rest))
	    },
	    [c @ ('a'..='z'|'A'..='Z'|'0'..='9'
		  |'/'|'.'|','|'*'|'?'|'$'|'^'|'-'|'_'),
	     rest @ ..] => Self::eat_basic_str(rest,(*c).into()),
	    [c,..] => bail!("Unexpected character {:?}",c),
	    [] => Ok((Self::Eof,u))
	}
    }

    fn parse_size(u:&[char])->Result<(u64,&[char])> {
	let (n,rest) = Self::parse_u64(u,String::new())?;
	match rest {
	    ['G',rest @ ..] => Ok((n << 30,rest)),
	    ['M',rest @ ..] => Ok((n << 20,rest)),
	    ['k',rest @ ..] => Ok((n << 10,rest)),
	    _ => Ok((n,rest))
	}
    }

    fn parse_u64(mut u:&[char],mut buf:String)->Result<(u64,&[char])> {
	loop {
	    match u {
		[c @ '0'..='9',rest @ ..] => {
		    buf.push(*c);
		    u = rest;
		},
		_ => {
		    let x : u64 = buf.parse()?;
		    return Ok((x,u))
		}
	    }
	}
    }

    fn parse_i32(u:&[char])->Result<i32> {
	let v : String = u.into_iter().collect();
	Ok(v.parse()?)
    }

    fn eat_keyword(mut u:&[char],mut buf:String)->Result<(String,&[char])> {
	loop {
	    match u {
		[c @ ('a'..='z'),
		 rest @ ..] => {
		    buf.push(*c);
		    u = rest;
		},
		_ => return Ok((buf,u))
	    }
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

    pub fn tokenize(u:&str)->Result<Vec<Self>> {
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
