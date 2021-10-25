mod errors;

use errors::{Res,error};
use pico_args::Arguments;
use serde::{Serialize,Deserialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::ffi::OsString;
use std::fs::File;
use std::io::{BufReader,BufWriter};
use std::os::unix::fs::MetadataExt;
use std::path::{Path,PathBuf};

#[derive(Debug,Serialize,Deserialize)]
pub struct Directory {
    entries:BTreeMap<OsString,Entry>
}

impl Directory {
    pub fn new()->Self {
	Self{ entries:BTreeMap::new() }
    }

    pub fn insert(&mut self,name:OsString,entry:Entry) {
	self.entries.insert(name,entry);
    }
}

#[derive(Debug,Serialize,Deserialize)]
pub enum Entry {
    Dir(Directory),
    File(u64),
    Other(u64),
    Error(String)
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Device {
    inodes:BTreeMap<u64,FileInfo>
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
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Mounts {
    devices:BTreeMap<u64,Device>
}

#[derive(Debug,Serialize,Deserialize)]
pub struct FileSystem {
    mounts:Mounts,
    tree:Entry
}

impl FileSystem {
    pub fn from_file<P:AsRef<Path>>(path:P)->Result<Self,Box<dyn Error>> {
        let fd = File::open(path)?;
        let mut buf = BufReader::new(fd);
        let fps : Self = rmp_serde::decode::from_read(&mut buf)?;
        Ok(fps)
    }

    pub fn save_to_file<P:AsRef<Path>>(&self,path:P)->Result<(),Box<dyn Error>> {
        let fd = File::create(path)?;
        let mut buf = BufWriter::new(fd);
        self.serialize(&mut rmp_serde::Serializer::new(&mut buf))?;
        Ok(())
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

    pub fn get_device(&mut self,dev:u64)->&mut Device {
	self.ensure_device(dev);
	self.devices.get_mut(&dev).unwrap()
    }
}

#[derive(Debug,Serialize,Deserialize)]
pub struct FileInfo {
    pub size:u64,
    pub modified:i64,
    pub accessed:i64,
    pub created:i64,
}

fn scan<P:AsRef<Path>+std::fmt::Debug>(mounts:&mut Mounts,path:P)->Res<Entry> {
    let mut dir = Directory::new();
    println!(">> {:?}",path);
    match std::fs::read_dir(&path) {
	Ok(rd) => {
	    for entry in rd {
		match entry {
		    Ok(e) => {
			let name = e.file_name();
			match e.metadata() {
			    Ok(md) => {
				let dev = md.dev();
				let d = mounts.get_device(dev);
				let ino = md.ino();
				if !d.has_inode(ino) {
				    let fi = FileInfo{
					size:md.size(),
					modified:md.mtime(),
					accessed:md.atime(),
					created:md.ctime()
				    };
				    d.insert_inode(ino,fi);
				}
				let ent =
				    if md.is_dir() {
					let mut sub_path = PathBuf::new();
					sub_path.push(&path);
					sub_path.push(&name);
					scan(mounts,sub_path)?
				    } else {
					if md.is_file() {
					    Entry::File(ino)
					} else {
					    Entry::Other(ino)
					}
				    };
				dir.insert(name,ent);
			    },
			    Err(err) => {
				eprintln!("Error getting metadata on {:?}: {:?}",name,err);
			    }
			}
		    },
		    Err(err) => {
			eprintln!("Error: {:?}",err);
		    }
		}
	    }
	    Ok(Entry::Dir(dir))
	},
	Err(e) => {
	    Ok(Entry::Error(e.to_string()))
	}
    }
}

fn collect(mut pargs:Arguments)->Res<()> {
    let path : OsString = pargs.value_from_str("--path")?;
    let out : OsString = pargs.value_from_str("--out")?;
    let mut mounts = Mounts::new();
    let tree = scan(&mut mounts,path)?;
    let fs = FileSystem{
	mounts,
	tree
    };
    fs.save_to_file(out)?;
    Ok(())
}

fn dump(mut pargs:Arguments)->Res<()> {
    let input : OsString = pargs.value_from_str("--in")?;
    let fs = FileSystem::from_file(input)?;
    println!("{:#?}",fs);
    Ok(())
}

fn main()->Res<()> {
    let mut pargs = Arguments::from_env();
    if pargs.contains("--collect") {
	collect(pargs)
    } else if pargs.contains("--dump") {
	dump(pargs)
    } else {
	Err(error("Bad arguments"))
    }
}
