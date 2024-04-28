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
    fn enter_device(&mut self,dev:u64)->Result<()> { Ok(()) }
    fn leave_device(&mut self)->Result<()> { Ok(()) }
    fn matching_entry(&mut self,
		      name:&OsString,
		      device:&Device,
		      entry:&Entry,
		      data:&FsData)->Result<()> { Ok(()) }
}
