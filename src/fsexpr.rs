use anyhow::Result;
use tz::UtcDateTime;
use regex::Regex;

use crate::boolean::Expr;

#[derive(Copy,Clone,Debug)]
pub struct FsDate {
    pub year:i32,
    pub month:i32,
    pub day:i32
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

pub type FsExpr = Expr<FsAtom>;

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

impl FsDate {
    pub fn to_timestamp(&self)->Result<i64> {
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
