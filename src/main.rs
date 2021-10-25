#![feature(is_symlink)]
mod errors;

use errors::{Res,error};
use pico_args::Arguments;
use serde::{Serialize,Deserialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::ffi::OsString;
use std::fs::{File,DirEntry};
use std::io::{BufReader,BufWriter,Write};
use std::os::unix::fs::MetadataExt;
use std::path::{Path,PathBuf};

#[derive(Debug,Serialize,Deserialize)]
pub struct Directory {
    dev:u64,
    entries:BTreeMap<OsString,Entry>
}

impl Directory {
    pub fn new(dev:u64)->Self {
	Self{ dev,entries:BTreeMap::new() }
    }

    pub fn insert(&mut self,name:OsString,entry:Entry) {
	self.entries.insert(name,entry);
    }
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

    pub fn get_inode(&self,ino:u64)->&FileInfo {
	self.inodes.get(&ino).expect("Cannot find inode")
    }
}

#[derive(Debug,Serialize,Deserialize)]
pub struct Mounts {
    devices:BTreeMap<u64,Device>
}

#[derive(Debug,Serialize,Deserialize)]
pub struct FileSystem {
    mounts:Mounts,
    root:Directory
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

    pub fn dump(&self) {
	self.dump_dir(&self.root,0);
    }

    fn put_indent(indent:usize) {
	for _ in 0..indent {
	    print!("  ");
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
		print!("{:10} ",fi.size);
		Self::put_indent(indent);
		println!("{}",name.to_string_lossy());
	    },
	    Entry::Dir(dir) => {
		print!("{:10} ","DIR");
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

#[derive(Debug,Serialize,Deserialize)]
pub struct FileInfo {
    pub size:u64,
    pub modified:i64,
    pub accessed:i64,
    pub created:i64,
}

trait Watcher {
    fn notify(&mut self,path:&Path);
    fn error(&mut self,path:&Path);
}

fn scan_entry<W:Watcher>(watcher:&mut W,mounts:&mut Mounts,path:&Path,e:&DirEntry)->Res<(Entry,OsString)> {
    let name = e.file_name();
    let md = e.metadata()?;
    let dev = md.dev();
    let d = mounts.get_device_mut(dev);
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
	    scan(watcher,mounts,&sub_path)?
	} else {
	    if md.is_file() {
		Entry::File(ino)
	    } else if md.is_symlink() {
		let pb = e.path().read_link()?;
		Entry::Symlink(pb.as_os_str().to_os_string())
	    } else {
		Entry::Other(ino)
	    }
	};
    Ok((ent,name))
}

fn scan<W:Watcher>(watcher:&mut W,mounts:&mut Mounts,path:&Path)->Res<Entry> {
    match path.symlink_metadata() {
	Ok(md) => {
	    let dev = md.dev();
	    let mut dir = Directory::new(dev);
	    watcher.notify(path);
	    match std::fs::read_dir(&path) {
		Ok(rd) => {
		    for entry in rd {
			match entry {
			    Ok(e) =>
				match scan_entry(watcher,mounts,path,&e) {
				    Ok((ent,name)) => dir.insert(name,ent),
				    Err(_) => watcher.error(path)
				},
			    Err(_) => watcher.error(path)
			}
		    }
		    Ok(Entry::Dir(dir))
		},
		Err(e) => {
		    watcher.error(path);
		    Ok(Entry::Error(e.to_string()))
		}
	    }
	},
	Err(e) => {
	    watcher.error(path);
	    Ok(Entry::Error(e.to_string()))
	}
    }
}


use std::time::Instant;

struct Valve {
    pub mask:u64,
    last:Instant,
    threshold:f64
}

impl Valve {
    pub fn new(threshold:f64)->Self {
	Self{
	    mask:1,
	    last:Instant::now(),
	    threshold
	}
    }

    pub fn tick(&mut self) {
	let now = Instant::now();
	let dur = now.duration_since(self.last);
	let dt = dur.as_secs_f64();
	if dt > 2.0 * self.threshold {
	    self.mask >>= 1;
	} else if dt < self.threshold / 2.0 {
	    self.mask = self.mask.wrapping_shl(1) | 1;
	}
	self.last = now;
    }
}

struct Counter {
    total:u64,
    errors:u64,
    count:u64,
    valve:Valve
}

impl Counter {
    pub fn new()->Self {
	Self{ total:0,errors:0,count:0,valve:Valve::new(0.1) }
    }

    fn tick(&mut self,path:&Path) {
	self.count += 1;
	if self.count & self.valve.mask == 0 {
	    self.valve.tick();
	    let u = path.to_string_lossy();
	    print!("\r{:8} {:8} {}\x1b[K",self.total,self.errors,u);
	    std::io::stdout().flush().unwrap();
	}
    }
}

impl Watcher for Counter {
    fn notify(&mut self,path:&Path) {
	self.total += 1;
	self.tick(path);
    }

    fn error(&mut self,path:&Path) {
	self.errors += 1;
	self.tick(path);
    }
}

impl Drop for Counter {
    fn drop(&mut self) {
	println!("\nTotal: {}, errors: {}\x1b[K",self.total,self.errors);
    }
}

fn collect(mut pargs:Arguments)->Res<()> {
    let path_os : OsString = pargs.value_from_str("--path")?;
    let out : OsString = pargs.value_from_str("--out")?;
    let mut mounts = Mounts::new();
    let path = Path::new(&path_os);
    let mut counter = Counter::new();
    let fs =
	match scan(&mut counter,&mut mounts,&path)? {
	    Entry::Dir(root) =>
		FileSystem{
		    mounts,
		    root
		},
	    _ => return Err(error("Not a directory"))
	};
    fs.save_to_file(out)?;
    Ok(())
}

fn dump(mut pargs:Arguments)->Res<()> {
    let input : OsString = pargs.value_from_str("--in")?;
    let fs = FileSystem::from_file(input)?;
    fs.dump();
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
