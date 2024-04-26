use anyhow::{Result,anyhow};
use std::ffi::OsString;
use log::{self,info};

use crate::{
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
	let us : Vec<&str> = u.split_ascii_whitespace().collect();
	match &us[..] {
	    [cmd@("find"|"findi"),pat] => {
		let mut limit = self.limit;
		let mut finder = Finder::new(sd);
		finder.do_find_multi(&self.fs,
					  pat,&mut limit,*cmd == "findi")?;
	    },
	    ["limit",l] => {
		self.limit = l.parse()?;
		info!("New limit {}",self.limit);
	    },
	    ["quit"] => {
		std::process::exit(0);
		// return Ok(true);
	    },
	    [] => (),
	    _ => return Err(anyhow!("Unknown command"))
	}
	Ok(false)
    }
}
