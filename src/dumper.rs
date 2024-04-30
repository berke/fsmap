use std::ffi::OsString;
use std::path::{PathBuf};
use anyhow::{Result};
use log::warn;

use crate::{
    fsexpr::{FsData,Predicate},
    fsmap::*,
    sigint_detector::SigintDetector,
    watcher::{Action,Watcher}
};

pub struct Dumper<'a,'b,'c,P,W> {
    sd:&'a SigintDetector,
    fss:&'b FileSystems,
    pred:&'c P,
    idrive:usize,
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
	    matching_bytes:0,
	    matching_entries:0,
	    watcher
	}
    }

    pub fn into_inner(self)->W {
	self.watcher
    }
	
    pub fn dump(&mut self)->Result<()> {
	for (ifs,fse) in self.fss.systems.iter().enumerate() {
	    if let Action::Enter = self.watcher.enter_fs(ifs,fse)? {
		self.idrive = ifs;
		self.dump_dir(fse,&fse.fs.root)?;
		self.watcher.leave_fs()?;
	    }
	}
	Ok(())
    }

    fn dump_dir(&mut self,fse:&FileSystemEntry,dir:&Directory)->Result<()> {
	if let Some(device) = fse.fs.mounts.get_device(dir.dev) {
	    for (name,entry) in dir.entries.iter() {
		if self.sd.interrupted() {
		    self.watcher.interrupted()?;
		}
		if let Action::Skip = self.dump_entry(fse,name,device,entry)? {
		    break;
		}
	    }
	} else {
	    self.watcher.device_not_found(dir.dev)?;
	}
	Ok(())
    }

    fn dump_entry(&mut self,
		  fse:&FileSystemEntry,
		  name:&OsString,
		  device:&Device,
		  entry:&Entry)->Result<Action> {
	self.current.push(name);
	let nsl = name.to_string_lossy();
	let path = self.current.as_os_str().to_string_lossy();

	let mut data = FsData {
	    drive:self.idrive as u64,
	    name:&nsl,
	    path:&path,
	    timestamp:None,
	    size:None
	};

	match entry {
	    &Entry::File(ino) => {
		if let Some(fi) = device.get_inode(ino) {
		    data.size = Some(fi.size);
		    data.timestamp = Some(fi.unix_time());
		} else {
		    warn!("Inode {} not found",ino);
		}
	    },
	    Entry::Dir(_) |
	    Entry::Symlink(_) |
	    Entry::Other(_) |
	    Entry::Error(_) => ()
	}

	let mut action = Action::Enter;
	let show = self.pred.test(&data);
	if show {
	    self.matching_entries += 1;
	    self.matching_bytes += data.size.unwrap_or(0);
	    action = self.watcher.matching_entry(
		fse,
		name,
		device,
		entry,
		&data)?;
	}
	if let Action::Enter = action {
	    if let Entry::Dir(dir) = entry {
		if let Action::Enter = self.watcher.enter_dir(name)? {
		    self.dump_dir(fse,dir)?;
		    self.watcher.leave_dir()?;
		}
	    }
	}
	self.current.pop();
	Ok(action)
    }
}
