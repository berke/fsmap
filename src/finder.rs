use anyhow::{Result,bail};
use std::ffi::{OsString};
use std::path::{Path,PathBuf};

use crate::{
    fsmap::*,
    fsexpr::Predicate,
    sigint_detector::SigintDetector
};

pub struct Finder<P> {
    sd:SigintDetector,
    pred:P
}

impl<P> Finder<P> where P:Predicate {
    fn do_find_dir(&mut self,
		   dir:&Directory,
		   path:&Path,
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
	    if true { // self.pred.test(&u) {
		println!("{}",u);
		*limit -= 1;
	    }
	    if let Entry::Dir(dir) = entry {
		let mut pb = PathBuf::from(path);
		pb.push(name);
		self.do_find_dir(dir,&pb,limit)?;
	    }
	}
	Ok(())
    }

    fn do_find(&mut self,fs:&FileSystem,limit:&mut usize)->Result<()> {
	let path = Path::new("/");
	self.do_find_dir(&fs.root,path,limit)?;
	Ok(())
    }

    pub fn do_find_multi(&mut self,fs:&[(OsString,FileSystem)],
			 limit:&mut usize)->Result<()> {
	for (path,fs) in fs.iter() {
	    println!("{:?}:",path);
	    self.do_find(fs,limit)?;
	}
	Ok(())
    }

    pub fn new(sd:SigintDetector,pred:P)->Self {
	Self { sd,pred }
    }
}
