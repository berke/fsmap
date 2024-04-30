use std::ffi::OsString;
use anyhow::{bail,Result};
use log::warn;

use crate::{
    fsexpr::{FsData},
    fsmap::*
};

#[derive(Debug,Copy,Clone)]
pub enum Action {
    Skip,
    Enter
}

impl Action {
    pub fn is_skip(&self)->bool {
	match self {
	    Self::Skip => true,
	    _ => false
	}
    }
}

pub trait Watcher {
    fn interrupted(&mut self)->Result<()> {
	bail!("Interrupted");
    }

    fn device_not_found(&mut self,dev:u64)->Result<()> {
	warn!("Cannot find device {}",dev);
	Ok(())
    }
    
    fn enter_dir(&mut self,_name:&OsString)->Result<Action> { Ok(Action::Enter) }
    fn leave_dir(&mut self)->Result<()> { Ok(()) }
    fn enter_fs(&mut self,_i:usize,_fse:&FileSystemEntry)->Result<Action> {
	Ok(Action::Enter)
    }
    fn leave_fs(&mut self)->Result<()> { Ok(()) }
    fn matching_entry(&mut self,
		      _fse:&FileSystemEntry,
		      _name:&OsString,
		      _device:&Device,
		      _entry:&Entry,
		      _data:&FsData)->Result<Action> { Ok(Action::Enter) }
}
