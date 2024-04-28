use std::ffi::OsString;
use anyhow::{Result};

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
		      fse:&FileSystemEntry,
		      name:&OsString,
		      device:&Device,
		      entry:&Entry,
		      data:&FsData)->Result<Action> {
	self.results.push(data.map(|x| x.to_string()));
	Ok(Action::Enter)
    }
}

impl EntryCollector {
    pub fn new()->Self {
	Self { results:Vec::new() }
    }
    
    pub fn print(&self) {
	for FsDataGen { drive,path,name,.. } in self.results.iter() {
	    println!("{}:{} {}",drive,path,name);
	}
    }
}
