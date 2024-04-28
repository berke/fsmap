use std::ffi::OsString;
use std::path::{Component,PathBuf};
use anyhow::{bail,Result};
use tz::{DateTime,TimeZoneRef};
use log::warn;

use crate::{
    fsexpr::{FsData,FsDataGen,Predicate},
    fsmap::*,
    sigint_detector::SigintDetector
};

pub trait Watcher {
    fn interrupted(&mut self)->Result<()> { Ok(()) }
    fn device_not_found(&mut self,dev:u64)->Result<()> { Ok(()) }
    fn enter_dir(&mut self,name:&OsString)->Result<()> { Ok(()) }
    fn leave_dir(&mut self)->Result<()> { Ok(()) }
    fn matching_entry(&mut self,
		      name:&OsString,
		      device:&Device,
		      entry:&Entry,
		      data:&FsData)->Result<()> { Ok(()) }
}

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

pub enum IndentMode {
    Numbered,
    Spaces
}

impl IndentMode {
    fn put_indent(&self,indent:usize) {
	match self {
	    IndentMode::Numbered => print!(" {:2} ",indent),
	    IndentMode::Spaces => {
		for _ in 0..indent {
		    print!("  ");
		}
	    }
	}
    }
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
		self.dump_dev(fs,name,device,entry)?;
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

pub struct BasicPrinter<'a> {
    tz:TimeZoneRef<'a>,
    indent:usize,
    indent_mode:IndentMode,
    idrive_shown:Option<usize>,
}

impl<'a> BasicPrinter<'a> {
    pub fn new()->Self {
	Self {
	    tz:TimeZoneRef::utc(),
	    indent:0,
	    indent_mode:IndentMode::Spaces,
	    idrive_shown:None
	}
    }

    fn show_dir(&mut self)->Result<()> {
	// if Some(self.idrive) != self.idrive_shown {
	//     println!("DRV {:?}",
	// 	     self.fss.systems[self.idrive].origin);
	//     self.idrive_shown = Some(self.idrive);
	// }
	// let c1 : Vec<Component> = self.last_dir.components().collect();
	// let c2 : Vec<Component> = self.dir.components().collect();
	// let m1 = c1.len();
	// let m2 = c2.len();
	// let mut match_so_far = true;
	// for i in 0..m2 {
	//     match_so_far =
	// 	match_so_far &&
	// 	i < m1 &&
	// 	c1[i] == c2[i];
	//     if !match_so_far {
	// 	print!("{:21} ","   ");
	// 	self.put_indent(i);
	// 	match c2[i] {
	// 	    Component::Normal(u) => println!("{}/",u.to_string_lossy()),
	// 	    _ => ()
	// 	}
	//     }
	// }
	// if !match_so_far {
	//     self.last_dir = self.dir.clone();
	// }
	Ok(())
    }

    fn put_indent(&self,indent:usize) {
    }
}

impl<'a> Watcher for BasicPrinter<'a> {
    fn interrupted(&mut self)->Result<()> {
	bail!("Interrupted")
    }

    fn device_not_found(&mut self,dev:u64)->Result<()> {
	warn!("Cannot find device {}",dev);
	Ok(())
    }

    fn matching_entry(&mut self,
		      name:&OsString,
		      device:&Device,
		      entry:&Entry,
		      data:&FsData)->Result<()> {
	match entry {
	    &Entry::Dir(_) => {
		print!("{:21} ","DIR");
		self.put_indent(self.indent);
		println!("{}",
			 data.name);
	    },
	    &Entry::File(ino) => {
		let fi = device.get_inode(ino);
		let dt = DateTime::from_timespec(
		    fi.unix_time(),
		    0,
		    self.tz)?;
		print!("{:10} {:04}-{:02}-{:02} ",
		       fi.size,
		       dt.year(),
		       dt.month(),
		       dt.month_day());
		self.put_indent(self.indent);
		println!("{}",data.name);
	    },
	    Entry::Symlink(sl) => {
		print!("{:21} ","SYML");
		self.put_indent(self.indent);
		println!("{} -> {:?}",data.name,sl);
	    },
	    Entry::Other(ino) => {
		print!("{:21} ","OTHER");
		self.put_indent(self.indent);
		println!("{} ino {}",data.name,ino);
	    },
	    Entry::Error(err) => {
		print!("{:21} ","ERROR");
		self.put_indent(self.indent);
		println!("{} : {}",data.name,err);
	    },
	}
	Ok(())
    }
}

pub type FsDataOwned = FsDataGen<String>;

pub struct ResultCollector {
    pub results:Vec<FsDataOwned>
}

impl Watcher for ResultCollector {
    fn matching_entry(&mut self,
		      name:&OsString,
		      device:&Device,
		      entry:&Entry,
		      data:&FsData)->Result<()> {
	self.results.push(data.map(|x| x.to_string()));
	Ok(())
    }
}

impl ResultCollector {
    pub fn new()->Self {
	Self { results:Vec::new() }
    }
    
    pub fn print(&self) {
	for FsDataGen { drive,path,name,.. } in self.results.iter() {
	    println!("{}:{} {}",drive,path,name);
	}
    }
}
