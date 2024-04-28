use std::ffi::OsString;
use anyhow::{bail,Result};
use tz::{DateTime,TimeZoneRef};
use log::warn;
use std::path::{Component,PathBuf};

use crate::{
    fsexpr::{FsData},
    fsmap::*,
    watcher::{Action,Watcher}
};

#[derive(Default)]
struct DirState {
    breadth:usize,
    entries:usize
}

pub struct LimiterSettings {
    pub max_depth:usize,
    pub max_breadth:usize,
    pub max_entries:usize,
}

impl Default for LimiterSettings {
    fn default()->Self {
	LimiterSettings {
	    max_depth:usize::MAX,
	    max_breadth:usize::MAX,
	    max_entries:usize::MAX,
	}
    }
}

pub struct Limiter<'a,W> {
    settings:&'a LimiterSettings,
    stack:Vec<DirState>,
    watcher:W
}

impl<'a,W> Limiter<'a,W> where W:Watcher {
    pub fn new(settings:&'a LimiterSettings,watcher:W)->Self {
	Self {
	    settings,
	    stack:Vec::new(),
	    watcher
	}
    }

    pub fn into_inner(self)->W {
	self.watcher
    }
}

impl<'a,W> Watcher for Limiter<'a,W> where W:Watcher {
    fn enter_fs(&mut self,ifs:usize,fse:&FileSystemEntry)->Result<Action> {
	let n = self.stack.len();
	if n + 1 < self.settings.max_depth {
	    self.stack.clear();
	    self.stack.push(DirState::default());
	    self.watcher.enter_fs(ifs,fse)
	} else {
	    Ok(Action::Skip)
	}
    }

    fn leave_fs(&mut self)->Result<()> {
	self.watcher.leave_fs()?;
	self.stack.pop();
	Ok(())
    }

    fn enter_dir(&mut self,name:&OsString)->Result<Action> {
	let mut res = Action::Enter;
	let n = self.stack.len();
	let mut state = &mut self.stack[n - 1];
	if state.breadth + 1 < self.settings.max_breadth
	    && n + 1 < self.settings.max_depth {
		state.breadth += 1;
		self.stack.push(DirState::default());
		if self.watcher.enter_dir(name)?.is_skip() {
		    self.leave_dir()?;
		    Ok(Action::Skip)
		} else {
		    Ok(Action::Enter)
		}
	    } else {
		Ok(Action::Skip)
	    }
    }

    fn leave_dir(&mut self)->Result<()> {
	self.watcher.leave_dir()?;
	self.stack.pop();
	Ok(())
    }

    fn matching_entry(&mut self,
		      fse:&FileSystemEntry,
		      name:&OsString,
		      device:&Device,
		      entry:&Entry,
		      data:&FsData)->Result<Action> {
	let n = self.stack.len();
	let mut state = &mut self.stack[n - 1];
	if state.entries + 1 < self.settings.max_entries {
	    state.entries += 1;
	    self.watcher.matching_entry(fse,name,device,entry,data)
	} else {
	    Ok(Action::Skip)
	}
    }
}
