use anyhow::{Result,anyhow,bail};
use pico_args::Arguments;
use serde::{Serialize,Deserialize};
use std::collections::{BTreeMap};
use std::ffi::OsString;
use std::fs::{File,DirEntry};
use std::io::{BufReader,BufWriter,Write};
use std::os::unix::fs::MetadataExt;
use std::path::{Path,PathBuf};
use log::{self,info,warn,error,debug,trace};
use regex::{Regex,RegexBuilder};

use crate::{
    scanner::{Scanner,Watcher},
    valve::Valve
};

pub struct Counter {
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
