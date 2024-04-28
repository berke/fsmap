use anyhow::{Result,bail};

use crate::{
    basic_printer::BasicPrinter,
    dumper::Dumper,
    entry_collector::EntryCollector,
    fsexpr::Expr,
    fsmap::*,
    sigint_detector::SigintDetector,
    watcher::Watcher
};

pub struct ExaminerCli {
    fss:FileSystems,
    max_depth:usize,
    max_breadth:usize,
    max_entries:usize,
    limit:usize
}

const HELP_TEXT : &str = include_str!("../data/help.txt");

impl ExaminerCli {
    pub fn new(fss:FileSystems)->Self {
	Self {
	    fss,
	    max_depth:usize::MAX,
	    max_breadth:usize::MAX,
	    max_entries:usize::MAX,
	    limit:1000
	}
    }

    fn process<W:Watcher>(&mut self,w:&str,watcher:W)->Result<W> {
	let sd = SigintDetector::new();
	let expr = Expr::parse(w)?;
	let limit = self.limit;
	// let bp = EntryCollector::new();
	let mut dp = Dumper::new(&sd,&self.fss,&expr,watcher);
	match dp.dump() {
	    Ok(()) => (),
	    Err(e) => println!("{}",e)
	}
	println!("Entries: {}",dp.matching_entries);
	println!("Bytes: {}",dp.matching_bytes);
	Ok(dp.into_inner())
    }

    pub fn handle_input(&mut self,u:&str)->Result<bool> {
	let u = u.trim();
	if let Some((v,w)) = u.split_once(' ') {
	    match v {
		"find" => self.process(w,EntryCollector::new())?.print(),
		"tree" => {
		    let mut bp = BasicPrinter::new();
		    bp.set_max_depth(self.max_depth);
		    bp.set_max_breadth(self.max_breadth);
		    bp.set_max_entries(self.max_entries);
		    let _ = self.process(w,bp)?;
		},
		"limit" => {
		    let l : usize = w.parse()?;
		    self.limit = l;
		},
		"maxdepth" | "maxd" => {
		    let d : usize = w.parse()?;
		    self.max_depth = d;
		},
		"maxbreadth" | "maxb" => {
		    let d : usize = w.parse()?;
		    self.max_breadth = d;
		},
		"maxent" | "maxe" => {
		    let d : usize = w.parse()?;
		    self.max_entries = d;
		},
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
		"maxdepth?" | "maxd?" => {
		    if self.max_depth == usize::MAX {
			println!("Unlimited");
		    } else {
			println!("{}",self.max_depth);
		    }
		},
		"maxbreadth?" | "maxb?" => {
		    if self.max_breadth == usize::MAX {
			println!("Unlimited");
		    } else {
			println!("{}",self.max_breadth);
		    }
		},
		"maxent?" | "maxe?" => {
		    if self.max_entries == usize::MAX {
			println!("Unlimited");
		    } else {
			println!("{}",self.max_entries);
		    }
		},
		"limit?" => {
		    if self.limit == usize::MAX {
			println!("Unlimited");
		    } else {
			println!("{}",self.limit);
		    }
		},
		"unlimited" => self.limit = usize::MAX,
		"quit" => std::process::exit(0),
		"help" => print!("{}",HELP_TEXT),
		"" => (),
		_ => bail!("Unknown command with no arguments")
	    }
	}
	Ok(false)
    }
}
