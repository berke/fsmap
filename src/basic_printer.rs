use std::ffi::OsString;
use std::path::{Component,PathBuf};
use anyhow::{bail,Result};
use tz::{DateTime,TimeZoneRef};
use log::warn;

use crate::{
    fsexpr::{FsData,FsDataGen,Predicate},
    fsmap::*,
    indent::IndentMode,
    sigint_detector::SigintDetector,
    watcher::Watcher
};

pub struct BasicPrinter<'a> {
    tz:TimeZoneRef<'a>,
    indent:usize,
    indent_mode:IndentMode,
    idrive_shown:Option<usize>,
}

impl<'a> BasicPrinter<'a> {
    pub fn new()->Self {
	Self {
	    tz:TimeZoneRef::utc(),
	    indent:0,
	    indent_mode:IndentMode::Spaces,
	    idrive_shown:None
	}
    }

    fn show_dir(&mut self)->Result<()> {
	// if Some(self.idrive) != self.idrive_shown {
	//     println!("DRV {:?}",
	// 	     self.fss.systems[self.idrive].origin);
	//     self.idrive_shown = Some(self.idrive);
	// }
	// let c1 : Vec<Component> = self.last_dir.components().collect();
	// let c2 : Vec<Component> = self.dir.components().collect();
	// let m1 = c1.len();
	// let m2 = c2.len();
	// let mut match_so_far = true;
	// for i in 0..m2 {
	//     match_so_far =
	// 	match_so_far &&
	// 	i < m1 &&
	// 	c1[i] == c2[i];
	//     if !match_so_far {
	// 	print!("{:21} ","   ");
	// 	self.put_indent(i);
	// 	match c2[i] {
	// 	    Component::Normal(u) => println!("{}/",u.to_string_lossy()),
	// 	    _ => ()
	// 	}
	//     }
	// }
	// if !match_so_far {
	//     self.last_dir = self.dir.clone();
	// }
	Ok(())
    }

    fn put_indent(&self,indent:usize) {
    }
}

impl<'a> Watcher for BasicPrinter<'a> {
    fn interrupted(&mut self)->Result<()> {
	bail!("Interrupted")
    }

    fn device_not_found(&mut self,dev:u64)->Result<()> {
	warn!("Cannot find device {}",dev);
	Ok(())
    }

    fn matching_entry(&mut self,
		      name:&OsString,
		      device:&Device,
		      entry:&Entry,
		      data:&FsData)->Result<()> {
	match entry {
	    &Entry::Dir(_) => {
		print!("{:21} ","DIR");
		self.put_indent(self.indent);
		println!("{}",
			 data.name);
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
		println!("{}",data.name);
	    },
	    Entry::Symlink(sl) => {
		print!("{:21} ","SYML");
		self.put_indent(self.indent);
		println!("{} -> {:?}",data.name,sl);
	    },
	    Entry::Other(ino) => {
		print!("{:21} ","OTHER");
		self.put_indent(self.indent);
		println!("{} ino {}",data.name,ino);
	    },
	    Entry::Error(err) => {
		print!("{:21} ","ERROR");
		self.put_indent(self.indent);
		println!("{} : {}",data.name,err);
	    },
	}
	Ok(())
    }
}
