use std::ffi::OsString;
use anyhow::{bail,Result};
use tz::{DateTime,TimeZoneRef};
use log::warn;
use std::path::{Component,PathBuf};

use crate::{
    fsexpr::{FsData},
    fsmap::*,
    indent::IndentMode,
    watcher::{Action,Watcher}
};

#[derive(Default)]
struct DirState {
    breadth:usize,
    entries:usize
}

pub struct BasicPrinter<'a> {
    tz:TimeZoneRef<'a>,
    indent:usize,
    indent_mode:IndentMode,
    ifs:Option<usize>,
    ifs_shown:Option<usize>,
    dir:PathBuf,
    last_dir:PathBuf,
    max_depth:usize,
    max_breadth:usize,
    max_entries:usize,
    stack:Vec<DirState>
}

impl<'a> BasicPrinter<'a> {
    pub fn new()->Self {
	Self {
	    tz:TimeZoneRef::utc(),
	    indent:0,
	    indent_mode:IndentMode::Spaces,
	    ifs:None,
	    ifs_shown:None,
	    dir:PathBuf::new(),
	    last_dir:PathBuf::new(),
	    max_depth:usize::MAX,
	    max_breadth:usize::MAX,
	    max_entries:usize::MAX,
	    stack:Vec::new()
	}
    }

    pub fn set_max_depth(&mut self,max_depth:usize) {
	self.max_depth = max_depth;
    }

    pub fn set_max_breadth(&mut self,max_breadth:usize) {
	self.max_breadth = max_breadth;
    }

    pub fn set_max_entries(&mut self,max_entries:usize) {
	self.max_entries = max_entries;
    }

    fn show_dir(&mut self,fse:&FileSystemEntry)->Result<()> {
	if self.ifs != self.ifs_shown {
	    println!("DRV {:?}",
		     fse.origin);
	    self.ifs_shown = self.ifs;
	}
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
	Ok(())
    }

    fn put_indent(&self,indent:usize) {
	self.indent_mode.put_indent(indent);
    }

    fn ellipsis(&self) {
	print!("{:21} ","");
	self.put_indent(self.indent);
	println!("...");
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

    fn enter_fs(&mut self,ifs:usize,fse:&FileSystemEntry)->Result<Action> {
	if 0 < self.max_depth {
	    self.ifs = Some(ifs);
	    self.indent = 0;
	    self.dir.clear();
	    self.last_dir.clear();
	    self.stack.push(DirState::default());
	    Ok(Action::Enter)
	} else {
	    Ok(Action::Skip)
	}
    }

    fn leave_fs(&mut self)->Result<()> {
	self.ifs = None;
	Ok(())
    }

    fn enter_dir(&mut self,name:&OsString)->Result<Action> {
	let n = self.stack.len();
	let mut state = &mut self.stack[n - 1];
	if state.breadth < self.max_breadth {
	    state.breadth += 1;
	} else {
	    self.ellipsis();
	    return Ok(Action::Skip);
	}

	if self.indent + 1 < self.max_depth {
	    self.dir.push(name);
	    self.stack.push(DirState::default());
	    self.indent += 1;
	    Ok(Action::Enter)
	} else {
	    self.ellipsis();
	    Ok(Action::Skip)
	}
    }

    fn leave_dir(&mut self)->Result<()> {
	self.dir.pop();
	self.stack.pop();
	self.indent -= 1;
	Ok(())
    }

    fn matching_entry(&mut self,
		      fse:&FileSystemEntry,
		      name:&OsString,
		      device:&Device,
		      entry:&Entry,
		      data:&FsData)->Result<Action> {
	let n = self.stack.len();
	let mut state = &mut self.stack[n - 1];
	if state.entries + 1 < self.max_entries {
	    state.entries += 1;
	} else {
	    self.ellipsis();
	    return Ok(Action::Skip);
	}
	self.show_dir(fse)?;
	match entry {
	    &Entry::Dir(_) => {
		print!("{:21} ","DIR");
		self.put_indent(self.indent);
		println!("{}",data.name);
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
	Ok(Action::Enter)
    }
}
