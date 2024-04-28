use std::ffi::OsString;
use std::path::{Component,PathBuf};
use anyhow::{bail,Result};
use tz::{DateTime,TimeZoneRef};
use log::warn;

use crate::{
    fsexpr::{FsData,Predicate},
    fsmap::*,
    sigint_detector::SigintDetector
};

pub struct Dumper<'a,'b,'c,'d,P> {
    sd:&'a SigintDetector,
    fs:&'b FileSystem,
    pred:&'c P,
    last_dir:PathBuf,
    dir:PathBuf,
    current:PathBuf,
    indent:usize,
    indent_mode:IndentMode,
    tz:TimeZoneRef<'d>,
    pub matching_bytes:u64,
    pub matching_entries:usize,
}

pub enum IndentMode {
    Numbered,
    Spaces
}

impl<'a,'b,'c,'d,P> Dumper<'a,'b,'c,'d,P> where P:Predicate {
    pub fn new(sd:&'a SigintDetector,fs:&'b FileSystem,pred:&'c P)->Self {
	Self {
	    fs,
	    sd,
	    pred,
	    current:PathBuf::new(),
	    last_dir:PathBuf::new(),
	    dir:PathBuf::new(),
	    indent:0,
	    indent_mode:IndentMode::Spaces,
	    tz:TimeZoneRef::utc(),
	    matching_bytes:0,
	    matching_entries:0
	}
    }
	
    pub fn dump(&mut self)->Result<()> {
	self.dump_dir(&self.fs.root)
    }

    fn put_indent(&self,indent:usize) {
	match self.indent_mode {
	    IndentMode::Numbered => print!(" {:2} ",indent),
	    IndentMode::Spaces => {
		for _ in 0..indent {
		    print!("  ");
		}
	    }
	}
    }

    fn dump_dir(&mut self,dir:&Directory)->Result<()> {
	if let Some(device) = self.fs.mounts.get_device(dir.dev) {
	    for (name,entry) in dir.entries.iter() {
		if self.sd.interrupted() {
		    bail!("Interrupted");
		}
		self.dump_dev(name,device,entry)?;
	    }
	} else {
	    warn!("Cannot find device {}",dir.dev);
	}
	Ok(())
    }

    fn show_dir(&mut self) {
	let c1 : Vec<Component> = self.last_dir.components().collect();
	let c2 : Vec<Component> = self.dir.components().collect();
	let m1 = c1.len();
	let m2 = c2.len();
	let mut match_so_far = true;
	for i in 0..m2 {
	    match_so_far =
		match_so_far &&
		i < m1 &&
		c1[i] == c2[i];
	    if !match_so_far {
		print!("{:21} ","   ");
		self.put_indent(i);
		match c2[i] {
		    Component::Normal(u) => println!("{}/",u.to_string_lossy()),
		    _ => ()
		}
	    }
	}
	if !match_so_far {
	    self.last_dir = self.dir.clone();
	}
    }

    fn dump_dev(&mut self,name:&OsString,device:&Device,entry:&Entry)
		->Result<()> {
	self.current.push(name);
	let nsl = name.to_string_lossy();
	let path = self.current.as_os_str().to_string_lossy();

	let mut data = FsData {
	    name:&nsl,
	    path:&path,
	    timestamp:0,
	    size:0
	};

	match entry {
	    &Entry::File(ino) => {
		let fi = device.get_inode(ino);
		data.size = fi.size;
		data.timestamp = fi.unix_time();
	    },
	    Entry::Symlink(sl) => {
	    },
	    Entry::Other(ino) => {
	    },
	    Entry::Error(err) => {
	    },
	    _ => ()
	}

	let show = self.pred.test(&data);
	if show {
	    self.matching_entries += 1;
	    self.matching_bytes += data.size;
	    self.show_dir();
	    match entry {
		&Entry::Dir(_) => {
		    print!("{:21} ","DIR");
		    self.put_indent(self.indent);
		    println!("{}",
			     nsl);
		},
		&Entry::File(ino) => {
		    let fi = device.get_inode(ino);
		    let dt = DateTime::from_timespec(
			fi.unix_time(),
			0,
			self.tz)?;
		    print!("{:10} {:04}-{:02}-{:02} ",
			   fi.size,
			   dt.year(),
			   dt.month(),
			   dt.month_day());
		    self.put_indent(self.indent);
		    println!("{}",nsl);
		},
		Entry::Symlink(sl) => {
		    print!("{:21} ","SYML");
		    self.put_indent(self.indent);
		    println!("{} -> {:?}",nsl,sl);
		},
		Entry::Other(ino) => {
		    print!("{:21} ","OTHER");
		    self.put_indent(self.indent);
		    println!("{} ino {}",nsl,ino);
		},
		Entry::Error(err) => {
		    print!("{:21} ","ERROR");
		    self.put_indent(self.indent);
		    println!("{} : {}",nsl,err);
		},
	    }
	}
	if let Entry::Dir(dir) = entry {
	    self.indent += 1;
	    self.dir.push(name);
	    self.dump_dir(dir)?;
	    self.dir.pop();
	    self.indent -= 1;
	}
	self.current.pop();
	Ok(())
    }
}
