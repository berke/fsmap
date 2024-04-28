use anyhow::{Result,bail};
use std::ffi::OsString;
use regex::Regex;

use crate::{
    dumper::{BasicPrinter,Dumper,ResultCollector},
    fsexpr::Expr,
    fsmap::*,
    finder::Finder,
    sigint_detector::SigintDetector
};

pub struct ExaminerCli {
    fss:FileSystems,
    limit:usize
}

const HELP_TEXT : &str = include_str!("../data/help.txt");

impl ExaminerCli {
    pub fn new(fss:FileSystems)->Self {
	Self {
	    fss,
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
		    let bp = ResultCollector::new();
		    let mut dp = Dumper::new(&sd,&self.fss,&expr,bp);
		    match dp.dump() {
			Ok(()) => (),
			Err(e) => println!("{}",e)
		    }
		    println!("Entries: {}",dp.matching_entries);
		    println!("Bytes: {}",dp.matching_bytes);
		    let rc = dp.into_inner();
		    rc.print();
		},
		"limit" => {
		    let l : usize = w.parse()?;
		    self.limit = l;
		},
		_ => bail!("Unknown command"),
	    }
	} else {
	    match u {
		"drives" => {
		    println!("Drives:");
		    for (idrive,FileSystemEntry { origin,.. }) in
			self.fss.systems.iter().enumerate() {
			    println!("  {:3} {:?}",
				     idrive,
				     origin);
			}
		},
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
