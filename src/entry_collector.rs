use std::ffi::OsString;
use std::path::{Component,PathBuf};
use anyhow::{bail,Result};
use tz::{DateTime,TimeZoneRef};
use log::warn;

use crate::{
    fsexpr::{FsData,FsDataGen,Predicate},
    fsmap::*,
    sigint_detector::SigintDetector,
    watcher::Watcher
};

pub type FsDataOwned = FsDataGen<String>;

pub struct EntryCollector {
    pub results:Vec<FsDataOwned>
}

impl Watcher for EntryCollector {
    fn matching_entry(&mut self,
		      name:&OsString,
		      device:&Device,
		      entry:&Entry,
		      data:&FsData)->Result<()> {
	self.results.push(data.map(|x| x.to_string()));
	Ok(())
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
