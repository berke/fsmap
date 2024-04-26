use anyhow::Result;
use serde::{Serialize,Deserialize};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs::File;
use std::io::{BufReader,BufWriter};
use std::path::Path;
use log::{self,info};

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

#[derive(Debug,Serialize,Deserialize)]
pub struct FileInfo {
    pub size:u64,
    pub time:i32
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

    pub fn get_device_mut(&mut self,dev:u64)->&mut Device {
	self.ensure_device(dev);
	self.devices.get_mut(&dev).unwrap()
    }

    pub fn get_device(&self,dev:u64)->&Device {
	self.devices.get(&dev).unwrap()
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

    pub fn dump(&self) {
	self.dump_dir(&self.root,0);
    }

    fn put_indent(indent:usize) {
	for _ in 0..indent {
	    print!(" ");
	}
    }

    pub fn dump_dir(&self,dir:&Directory,indent:usize) {
	for (name,entry) in dir.entries.iter() {
	    self.dump_dev(name,dir.dev,entry,indent + 1);
	}
    }

    pub fn dump_dev(&self,name:&OsString,dev:u64,entry:&Entry,indent:usize) {
	match entry {
	    &Entry::File(ino) => {
		let fi = self.mounts.get_device(dev).get_inode(ino);
		print!("{:10} {:10} ",fi.size,fi.time);
		Self::put_indent(indent);
		println!("{}",name.to_string_lossy());
	    },
	    Entry::Dir(dir) => {
		print!("{:21} ","DIR");
		Self::put_indent(indent);
		println!("{}",name.to_string_lossy());
		self.dump_dir(dir,indent + 1);
	    },
	    Entry::Symlink(sl) => println!(" -> {:?}",sl),
	    Entry::Other(ino) => println!(" OTHER {}",ino),
	    Entry::Error(err) => println!(" ERROR {}",err)
	}
    }
}
