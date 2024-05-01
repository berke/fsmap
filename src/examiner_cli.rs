use anyhow::{Result,bail};

use crate::{
    basic_printer::BasicPrinter,
    dumper::Dumper,
    fsexpr::FsExpr,
    fsmap::*,
    help,
    indent::IndentMode,
    limiter::{Limiter,LimiterSettings},
    list_printer::ListPrinter,
    sigint_detector::SigintDetector,
    watcher::Watcher
};

pub struct ExaminerCli {
    fss:FileSystems,
    limiter:LimiterSettings,
    show_counts:bool
}


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
	let expr =
	    if w.is_empty() {
		FsExpr::True
	    } else {
		FsExpr::parse(w)?
	    };
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
	let (v,w) =
	    u.split_once(' ')
	    .unwrap_or((u,""));
	match v {
	    "list" | "ls" => {
		let bp = ListPrinter::new(false);
		let _ = self.process(w,bp)?;
	    },
	    "longlist" | "ll" => {
		let bp = ListPrinter::new(true);
		let _ = self.process(w,bp)?;
	    },
	    "tree" | "tr" => {
		let bp = BasicPrinter::new();
		let _ = self.process(w,bp)?;
	    },
	    "ntree" | "ntr" => {
		let mut bp = BasicPrinter::new();
		bp.set_indent_mode(IndentMode::Numbered);
		let _ = self.process(w,bp)?;
	    },
	    "maxdepth" | "maxd" =>
		Self::set_limit(w,&mut self.limiter.max_depth)?,
	    "maxbreadth" | "maxb" =>
		Self::set_limit(w,&mut self.limiter.max_breadth)?, 
	    "maxent" | "maxe" =>
		Self::set_limit(w,&mut self.limiter.max_entries)?,
	    "drives" => {
		println!("Drives:");
		for (idrive,FileSystemEntry { origin,.. }) in
		    self.fss.systems.iter().enumerate() {
			println!("  {:3} {:?}",
				 idrive,
				 origin);
		    }
	    },
	    "counts" => {
		self.show_counts = true;
		println!("Will show counts");
	    },
	    "nocounts" => {
		self.show_counts = false;
		println!("Won't show counts");
	    },
	    "maxdepth?" | "maxd?" => self.show_limit(self.limiter.max_depth),
	    "maxbreadth?" | "maxb?" => self.show_limit(self.limiter.max_breadth),
	    "maxent?" | "maxe?" => self.show_limit(self.limiter.max_entries),
	    "quit" => return Ok(true),
	    "help" | "h" => print!("{}",help::CLI_TEXT),
	    "help-expr" | "he" => print!("{}",help::EXPR_TEXT),
	    "" => (),
	    _ => bail!("Unknown command")
	}
	Ok(false)
    }
}
