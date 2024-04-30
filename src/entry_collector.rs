use std::ffi::OsString;
use anyhow::{bail,Result};

use crate::{
    fsexpr::{FsData,FsDataGen},
    fsmap::*,
    watcher::{Action,Watcher}
};

pub type FsDataOwned = FsDataGen<String>;

pub struct EntryCollector {
    pub results:Vec<FsDataOwned>
}

impl Watcher for EntryCollector {
    fn matching_entry(&mut self,
		      _fse:&FileSystemEntry,
		      _name:&OsString,
		      _device:&Device,
		      _entry:&Entry,
		      data:&FsData)->Result<Action> {
	self.results.push(data.map(|x| x.to_string()));
	Ok(Action::Enter)
    }

    fn interrupted(&mut self)->Result<()> {
	bail!("Interrupted");
    }
}

impl EntryCollector {
    pub fn new()->Self {
	Self { results:Vec::new() }
    }
    
    pub fn print(&self) {
	for FsDataGen { drive,path,.. } in self.results.iter() {
	    println!("{}:{}",drive,path);
	}
    }
}
