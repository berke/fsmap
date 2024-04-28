use anyhow::{Error,Result};
use serde::{Serialize,Deserialize};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs::{File,Metadata};
use std::io::{BufReader,BufWriter};
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use log::{self,error,info};

#[derive(Debug,Serialize,Deserialize)]
pub struct Directory {
    pub dev:u64,
    pub entries:Vec<(OsString,Entry)>
}

#[derive(Debug,Serialize,Deserialize)]
pub enum Entry {
    Dir(Directory),
    File(u64),
    Symlink(OsString),
    Other(u64),
    Error(String)
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Device {
    pub inodes:BTreeMap<u64,FileInfo>
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Mounts {
    pub devices:BTreeMap<u64,Device>
}

#[derive(Debug,Serialize,Deserialize)]
pub struct FileSystem {
    pub mounts:Mounts,
    pub root:Directory
}

#[derive(Debug)]
pub struct FileSystemEntry {
    pub origin:OsString,
    pub fs:FileSystem
}

#[derive(Debug)]
pub struct FileSystems {
    pub systems:Vec<FileSystemEntry>
}

#[derive(Debug,Serialize,Deserialize)]
pub struct FileInfo {
    pub size:u64,
    pub time:i32
}

impl FileInfo {
    pub fn of_metadata(md:&Metadata)->Self {
	let time = (md.mtime().max(md.atime()).max(md.ctime()) / 60) as i32;
	Self {
	    size:md.size(),
	    time
	}
    }

    pub fn unix_time(&self)->i64 {
	self.time as i64 * 60
    }
}

impl Device {
    pub fn new()->Self {
	Self{ inodes:BTreeMap::new() }
    }

    pub fn has_inode(&mut self,ino:u64)->bool {
	self.inodes.contains_key(&ino)
    }

    pub fn insert_inode(&mut self,ino:u64,fi:FileInfo) {
	self.inodes.insert(ino,fi);
    }

    pub fn get_inode(&self,ino:u64)->&FileInfo {
	self.inodes.get(&ino).expect("Cannot find inode")
    }
}

impl Directory {
    pub fn new(dev:u64)->Self {
	Self{ dev,entries:vec![] }
    }

    pub fn insert(&mut self,name:OsString,entry:Entry) {
	self.entries.push((name,entry));
    }
}

impl Mounts {
    pub fn new()->Self {
	Self{ devices:BTreeMap::new() }
    }

    pub fn ensure_device(&mut self,dev:u64) {
	if self.devices.contains_key(&dev) {
	    return;
	}
	self.devices.insert(dev,Device::new());
    }

    pub fn get_device_mut(&mut self,dev:u64)->Option<&mut Device> {
	self.ensure_device(dev);
	self.devices.get_mut(&dev)
    }

    pub fn get_device(&self,dev:u64)->Option<&Device> {
	self.devices.get(&dev)
    }
}

impl FileSystem {
    pub fn from_file<P:AsRef<Path>>(path:P)->Result<Self> {
	info!("Loading {:?}...",path.as_ref());
        let fd = File::open(path)?;
        let mut buf = BufReader::new(fd);
        let fps : Self = rmp_serde::decode::from_read(&mut buf)?;
        Ok(fps)
    }

    pub fn save_to_file<P:AsRef<Path>>(&self,path:P)->Result<()> {
        let fd = File::create(path)?;
        let mut buf = BufWriter::new(fd);
        self.serialize(&mut rmp_serde::Serializer::new(&mut buf))?;
        Ok(())
    }
}

impl FileSystems {
    pub fn load_multiple<P:AsRef<Path>>(paths:&[P])->
	(Self,Vec<(OsString,Error)>) {
	let mut systems = Vec::new();
	let mut errors = Vec::new();
	for p in paths.iter() {
	    let name = p.as_ref().as_os_str().to_os_string();
	    match FileSystem::from_file(p) {
		Ok(fs) => systems.push(FileSystemEntry { origin:name,fs }),
		Err(e) => {
		    error!("Error loading {:?}: {}",name,e);
		    errors.push((name,e));
		}
	    }
	}
	(Self { systems },errors)
    }
}
