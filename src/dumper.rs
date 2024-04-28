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

pub struct Dumper<'a,'b,'c,P,W> {
    sd:&'a SigintDetector,
    fss:&'b FileSystems,
    pred:&'c P,
    idrive:usize,
    last_dir:PathBuf,
    dir:PathBuf,
    current:PathBuf,
    watcher:W,
    pub matching_bytes:u64,
    pub matching_entries:usize,
}

impl<'a,'b,'c,P,W> Dumper<'a,'b,'c,P,W> where P:Predicate,W:Watcher {
    pub fn new(sd:&'a SigintDetector,
	       fss:&'b FileSystems,
	       pred:&'c P,
	       watcher:W)->Self {
	Self {
	    fss,
	    sd,
	    pred,
	    idrive:0,
	    current:PathBuf::new(),
	    last_dir:PathBuf::new(),
	    dir:PathBuf::new(),
	    matching_bytes:0,
	    matching_entries:0,
	    watcher
	}
    }

    pub fn into_inner(self)->W {
	self.watcher
    }
	
    pub fn dump(&mut self)->Result<()> {
	for (idrive,FileSystemEntry { fs,.. } ) in self.fss.systems.iter().enumerate() {
	    self.idrive = idrive;
	    self.dump_dir(&fs,&fs.root)?;
	}
	Ok(())
    }

    fn dump_dir(&mut self,fs:&FileSystem,dir:&Directory)->Result<()> {
	if let Some(device) = fs.mounts.get_device(dir.dev) {
	    for (name,entry) in dir.entries.iter() {
		if self.sd.interrupted() {
		    self.watcher.interrupted()?;
		}
		self.watcher.enter_device(dir.dev)?;
		self.dump_dev(fs,name,device,entry)?;
		self.watcher.leave_device()?;
	    }
	} else {
	    self.watcher.device_not_found(dir.dev)?;
	}
	Ok(())
    }

    fn dump_dev(&mut self,fs:&FileSystem,name:&OsString,device:&Device,entry:&Entry)
		->Result<()> {
	self.current.push(name);
	let nsl = name.to_string_lossy();
	let path = self.current.as_os_str().to_string_lossy();

	let mut data = FsData {
	    drive:self.idrive as u64,
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
	    self.watcher.matching_entry(
		name,
		device,
		entry,
		&data)?;
	}
	if let Entry::Dir(dir) = entry {
	    self.dir.push(name);
	    self.watcher.enter_dir(name)?;
	    self.dump_dir(fs,dir)?;
	    self.watcher.leave_dir()?;
	    self.dir.pop();
	}
	self.current.pop();
	Ok(())
    }
}
