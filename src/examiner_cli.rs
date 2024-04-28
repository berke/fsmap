use anyhow::{Result,bail};
use std::ffi::OsString;
use regex::Regex;

use crate::{
    dumper::Dumper,
    fsexpr::Expr,
    fsmap::*,
    finder::Finder,
    sigint_detector::SigintDetector
};

pub struct ExaminerCli {
    fs:Vec<(OsString,FileSystem)>,
    limit:usize
}

const HELP_TEXT : &str = include_str!("../data/help.txt");

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
		    let expr = Expr::parse(w)?;
		    let mut limit = self.limit;
		    for (drive,fs) in self.fs.iter() {
			let mut dp = Dumper::new(&sd,&fs,&expr);
			dp.dump()?;
		    }
		    // let mut finder = Finder::new(sd,x);
		    // finder.do_find_multi(
		    // 	&self.fs,
		    // 	&mut limit)?;
		},
		"limit" => {
		    let l : usize = w.parse()?;
		    self.limit = l;
		},
		_ => bail!("Unknown command"),
	    }
	} else {
	    match u {
		"limit?" => {
		    if self.limit == usize::MAX {
			println!("Unlimited");
		    } else {
			println!("{}",self.limit);
		    }
		},
		"unlimited" => self.limit = usize::MAX,
		"quit" => std::process::exit(0),
		"help" => print!("{}",HELP_TEXT),
		"" => (),
		_ => bail!("Unknown command with no arguments")
	    }
	}
	Ok(false)
    }
}
