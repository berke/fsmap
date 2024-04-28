use std::ffi::OsString;
use anyhow::{Result};

use crate::{
    fsexpr::{FsData},
    fsmap::*
};

#[derive(Debug,Copy,Clone)]
pub enum Action {
    Skip,
    Enter
}

pub trait Watcher {
    fn interrupted(&mut self)->Result<()> { Ok(()) }
    fn device_not_found(&mut self,dev:u64)->Result<()> { Ok(()) }
    fn enter_dir(&mut self,name:&OsString)->Result<Action> { Ok(Action::Enter) }
    fn leave_dir(&mut self)->Result<()> { Ok(()) }
    fn enter_fs(&mut self,i:usize,fse:&FileSystemEntry)->Result<Action> { Ok(Action::Enter) }
    fn leave_fs(&mut self)->Result<()> { Ok(()) }
    fn matching_entry(&mut self,
		      fse:&FileSystemEntry,
		      name:&OsString,
		      device:&Device,
		      entry:&Entry,
		      data:&FsData)->Result<Action> { Ok(Action::Enter) }
}
