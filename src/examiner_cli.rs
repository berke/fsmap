use anyhow::{Result,bail};

use crate::{
    basic_printer::BasicPrinter,
    dumper::Dumper,
    entry_collector::EntryCollector,
    fsexpr::Expr,
    fsmap::*,
    limiter::{Limiter,LimiterSettings},
    sigint_detector::SigintDetector,
    watcher::Watcher
};

pub struct ExaminerCli {
    fss:FileSystems,
    limiter:LimiterSettings,
    show_counts:bool
}

const HELP_TEXT : &str = include_str!("../data/help.txt");

impl ExaminerCli {
    pub fn new(fss:FileSystems)->Self {
	Self {
	    fss,
	    limiter:LimiterSettings::default(),
	    show_counts:false
	}
    }

    fn process<W:Watcher>(&mut self,w:&str,watcher:W)->Result<W> {
	let sd = SigintDetector::new();
	let expr = Expr::parse(w)?;
	let lim = Limiter::new(&self.limiter,watcher);
	let mut dp = Dumper::new(&sd,&self.fss,&expr,lim);
	match dp.dump() {
	    Ok(()) => (),
	    Err(e) => println!("{}",e)
	}
	if self.show_counts {
	    println!("Entries: {}",dp.matching_entries);
	    println!("Bytes: {}",dp.matching_bytes);
	}
	Ok(dp.into_inner().into_inner())
    }

    fn show_limit(&self,d:usize) {
	if d == usize::MAX {
	    println!("unlimited");
	} else {
	    println!("{}",d);
	}
    }

    fn set_limit(w:&str,l:&mut usize)->Result<()> {
	if w == "u" {
	    *l = usize::MAX;
	} else {
	    let d : usize = w.parse()?;
	    *l = d;
	}
	Ok(())
    }

    pub fn handle_input(&mut self,u:&str)->Result<bool> {
	let u = u.trim();
	if let Some((v,w)) = u.split_once(' ') {
	    match v {
		"list" | "ls" => self.process(w,EntryCollector::new())?.print(),
		"tree" | "tr" => {
		    let mut bp = BasicPrinter::new();
		    let _ = self.process(w,bp)?;
		},
		"maxdepth" | "maxd" =>
		    Self::set_limit(w,&mut self.limiter.max_depth)?,
		"maxbreadth" | "maxb" =>
		    Self::set_limit(w,&mut self.limiter.max_breadth)?, 
		"maxent" | "maxe" =>
		    Self::set_limit(w,&mut self.limiter.max_entries)?,
		_ => bail!("Unknown command"),
	    }
	} else {
	    match u {
		"drives" => {
		    println!("Drives:");
		    for (idrive,FileSystemEntry { origin,.. }) in
			self.fss.systems.iter().enumerate() {
			    println!("  {:3} {:?}",
				     idrive,
				     origin);
			}
		},
		"counts" => self.show_counts = true,
		"nocounts" => self.show_counts = false,
		"maxdepth?" | "maxd?" => self.show_limit(self.limiter.max_depth),
		"maxbreadth?" | "maxb?" => self.show_limit(self.limiter.max_breadth),
		"maxent?" | "maxe?" => self.show_limit(self.limiter.max_entries),
		"quit" => std::process::exit(0),
		"help" => print!("{}",HELP_TEXT),
		"" => (),
		_ => bail!("Unknown command with no arguments")
	    }
	}
	Ok(false)
    }
}
