use anyhow::{Result,bail};
use std::ffi::OsString;
use std::path::{Path,PathBuf};
use regex::{Regex,RegexBuilder};

use crate::{
    fsmap::*,
    sigint_detector::SigintDetector
};

pub struct Finder {
    sd:SigintDetector
}

impl Finder {
    fn do_find_dir(&mut self,
		   fs:&FileSystem,dir:&Directory,re:&Regex,path:&Path,
		   limit:&mut usize)->Result<()> {
	for (name,entry) in dir.entries.iter() {
	    if *limit == 0 {
		return Ok(());
	    }
	    if self.sd.interrupted() {
		bail!("Interrupted");
	    }
	    let mut pb = PathBuf::from(path);
	    pb.push(name);
	    let u = pb.as_os_str().to_string_lossy();
	    if re.is_match(&u) {
		println!("{}",u);
		*limit -= 1;
	    }
	    match entry {
		Entry::Dir(dir) => {
		    let mut pb = PathBuf::from(path);
		    pb.push(name);
		    self.do_find_dir(fs,dir,re,&pb,limit)?;
		},
		_ => ()
	    }
	}
	Ok(())
    }

    fn do_find(&mut self,fs:&FileSystem,pat:&str,limit:&mut usize,case:bool)->Result<()> {
	let re = RegexBuilder::new(pat).case_insensitive(case).build()?;
	let path = Path::new("/");
	self.do_find_dir(fs,&fs.root,&re,&path,limit)?;
	Ok(())
    }

    pub fn do_find_multi(&mut self,fs:&[(OsString,FileSystem)],pat:&str,
		     limit:&mut usize,case:bool)->Result<()> {
	for (path,fs) in fs.iter() {
	    println!("{:?}:",path);
	    self.do_find(fs,pat,limit,case)?;
	}
	Ok(())
    }

    pub fn new(sd:SigintDetector)->Self {
	Self { sd }
    }
}
