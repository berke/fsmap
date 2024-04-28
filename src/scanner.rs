use anyhow::{anyhow,Result};
use std::ffi::OsString;
use std::fs::{DirEntry};
use std::os::unix::fs::MetadataExt;
use std::path::{Path,PathBuf};

use crate::{
    fsmap::*
};

pub trait Watcher {
    fn notify(&mut self,path:&Path);
    fn error(&mut self,path:&Path);
}

pub struct Scanner<W> {
    watcher:W,
    one_device:bool,
    device:Option<u64>
}

impl<W> Scanner<W> where W:Watcher {
    pub fn new(watcher:W,one_device:bool)->Self {
	Self {
	    watcher,
	    one_device,
	    device:None
	}
    }
    
    fn scan_entry(&mut self,mounts:&mut Mounts,path:&Path,e:&DirEntry)->Result<(Entry,OsString)> {
	let name = e.file_name();
	let mut sub_path = PathBuf::new();
	sub_path.push(path);
	sub_path.push(&name);
	self.watcher.notify(&sub_path);
	let md = e.metadata()?;
	let dev = md.dev();
	let d = mounts.get_device_mut(dev)
	    .ok_or_else(|| anyhow!("Cannot find device"))?;
	let ino = md.ino();
	if !d.has_inode(ino) {
	    let fi = FileInfo::of_metadata(&md);
	    d.insert_inode(ino,fi);
	}
	let ent =
	    if md.is_dir() {
		self.scan(mounts,&sub_path)?
	    } else if md.is_file() {
		Entry::File(ino)
	    } else if md.is_symlink() {
		let pb = e.path().read_link()?;
		Entry::Symlink(pb.as_os_str().to_os_string())
	    } else {
		Entry::Other(ino)
	    };
	Ok((ent,name))
    }

    pub fn scan(&mut self,mounts:&mut Mounts,path:&Path)->Result<Entry> {
	match path.symlink_metadata() {
	    Ok(md) => {
		let dev = md.dev();
		if self.one_device &&
		     self.device.map(|dev2| dev != dev2).unwrap_or(false) {
		    return Ok(Entry::Error(
			format!("Skip dev {} {:?}",dev,path)));
		}
		self.device = Some(dev);
		let mut dir = Directory::new(dev);
		self.watcher.notify(path);
		match std::fs::read_dir(path) {
		    Ok(rd) => {
			for entry in rd {
			    match entry {
				Ok(e) =>
				    match self.scan_entry(mounts,path,&e) {
					Ok((ent,name)) => dir.insert(name,ent),
					Err(_) => self.watcher.error(path)
				    },
				Err(_) => self.watcher.error(path)
			    }
			}
			Ok(Entry::Dir(dir))
		    },
		    Err(e) => {
			self.watcher.error(path);
			Ok(Entry::Error(e.to_string()))
		    }
		}
	    },
	    Err(e) => {
		self.watcher.error(path);
		Ok(Entry::Error(e.to_string()))
	    }
	}
    }
}
