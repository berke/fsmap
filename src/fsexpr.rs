use anyhow::{bail,Result};
use tz::UtcDateTime;
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

#[derive(Copy,Clone,Debug)]
pub struct FsDate {
    pub year:i32,
    pub month:i32,
    pub day:i32
}

impl FsDate {
    fn to_timestamp(&self)->Result<i64> {
	Ok(UtcDateTime::new(
	    self.year,
	    self.month as u8,
	    self.day as u8,
	    0,
	    0,
	    0,
	    0)?
	    .unix_time())
    }
}

#[derive(Clone,Debug)]
pub enum FsAtom {
    Drive(u64),
    PathMatch(Regex),
    NameMatch(Regex),
    Before(i64),
    After(i64),
    Smaller(u64),
    Larger(u64)
}

pub struct FsDataGen<T> {
    // Drive ID
    pub drive:u64,
    
    // Base name
    pub name:T,
    
    // Full path
    pub path:T,

    // Timestamp (Unix)
    pub timestamp:i64,

    // Size (bytes)
    pub size:u64,
}

impl<T> FsDataGen<T> {
    pub fn map<U,F:Fn(&T)->U>(&self,f:F)->FsDataGen<U> {
	let &Self { drive,ref name,ref path,timestamp,size } = self;
	FsDataGen {
	    name:f(name),
	    path:f(path),
	    drive,
	    timestamp,
	    size
	}
    }
}

pub type FsData<'a> = FsDataGen<&'a str>;

pub trait Predicate {
    fn test(&self,data:&FsData)->bool;
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

impl FsAtom {
    pub fn eval(&self,data:&FsData)->bool {
	match self {
	    &Self::Drive(x) => data.drive == x,
	    Self::PathMatch(rx) => rx.is_match(data.path),
	    Self::NameMatch(rx) => rx.is_match(data.name),
	    &Self::Smaller(x) => data.size <= x,
	    &Self::Larger(x) => x <= data.size,
	    &Self::Before(x) => data.timestamp <= x,
	    &Self::After(x) => x <= data.timestamp,
	}
    }
}

impl Predicate for Expr<FsAtom> {
    fn test(&self,data:&FsData)->bool {
	self.eval(&|atom:&FsAtom| atom.eval(data))
    }
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
