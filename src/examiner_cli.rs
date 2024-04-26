use anyhow::{Result,bail};
use std::ffi::OsString;
use regex::Regex;

use crate::{
    fsexpr::Expr,
    fsmap::*,
    finder::Finder,
    sigint_detector::SigintDetector
};

pub struct ExaminerCli {
    fs:Vec<(OsString,FileSystem)>,
    limit:usize
}

impl ExaminerCli {
    pub fn new(fs:Vec<(OsString,FileSystem)>)->Self {
	Self {
	    fs,
	    limit:1000
	}
    }

    pub fn handle_input(&mut self,u:&str)->Result<bool> {
	let sd = SigintDetector::new();
	let u = u.trim();
	if let Some((v,w)) = u.split_once(' ') {
	    match v {
		"find" => {
		    let x : Expr<Regex> = Expr::parse(w)?;
		    let mut limit = self.limit;
		    let mut finder = Finder::new(sd,x);
		    finder.do_find_multi(
			&self.fs,
			&mut limit)?;
		},
		_ => bail!("Unknown command"),
	    }
	} else {
	    match u {
		"quit" => std::process::exit(0),
		"help" => {
		    println!("Commands:\n\
			      \n\
			      find EXPR\n\
			      'REGEX'\n\
			      quit\n\
			      help\n\
			      \n\
			      Grammar:\n\
			      \n\
			      'REGEX'\n\
			      REGEX\n\
			      EXPR & EXPR\n\
			      EXPR | EXPR\n\
			      EXPR \\ EXPR\n\
			      ( EXPR )\n\
			      %t\n\
			      %f");
		},
		"" => (),
		_ => bail!("Unknown command with no arguments")
	    }
	}
	Ok(false)
    }
}
