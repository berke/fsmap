use std::ffi::OsString;
use anyhow::Result;
use tz::{DateTime,TimeZoneRef};

use crate::{
    fsexpr::{FsData},
    fsmap::*,
    watcher::{Action,Watcher}
};

pub struct ListPrinter<'a> {
    tz:TimeZoneRef<'a>,
    long:bool
}

impl<'a> ListPrinter<'a> {
    pub fn new(long:bool)->Self {
	Self {
	    tz:TimeZoneRef::utc(),
	    long
	}
    }
}

impl<'a> Watcher for ListPrinter<'a> {
    fn matching_entry(&mut self,
		      _fse:&FileSystemEntry,
		      _name:&OsString,
		      device:&Device,
		      entry:&Entry,
		      data:&FsData)->Result<Action> {
	print!("{}:{}",data.drive,data.path);
	if self.long {
	    match entry {
		&Entry::Dir(_) => {
		    print!("/");
		},
		&Entry::File(ino) => {
		    if let Some(fi) = device.get_inode(ino) {
			let dt = DateTime::from_timespec(
			    fi.unix_time(),
			    0,
			    self.tz)?;
			print!(" {} {:04}-{:02}-{:02}",
			       fi.size,
			       dt.year(),
			       dt.month(),
			       dt.month_day());
		    } else {
			print!(" NO-INODE {}",ino);
		    }
		},
		Entry::Symlink(sl) => {
		    print!(" -> {:?}",sl);
		},
		Entry::Other(ino) => {
		    print!(" OTHER {}",ino);
		},
		Entry::Error(err) => {
		    print!(" ERROR {}",err);
		},
	    }
	}
	println!();
	Ok(Action::Enter)
    }
}
