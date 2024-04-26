use anyhow::{Result,anyhow,bail};
use pico_args::Arguments;
use std::ffi::OsString;
use std::path::{Path,PathBuf};
use log::{self,info};
use regex::{Regex,RegexBuilder};

mod counter;
mod fsmap;
mod scanner;
mod sigint_detector;
mod valve;

use fsmap::*;
use sigint_detector::SigintDetector;
use counter::Counter;
use scanner::Scanner;

fn collect(mut pargs:Arguments)->Result<()> {
    let path_os : OsString = pargs.value_from_str("--path")?;
    let out : OsString = pargs.value_from_str("--out")?;
    let one_device : bool = pargs.contains("--one-device");
    let mut mounts = Mounts::new();
    let path = Path::new(&path_os);
    let counter = Counter::new();
    let mut scanner = Scanner::new(counter,one_device);
    let fs =
	match scanner.scan(&mut mounts,&path)? {
	    Entry::Dir(root) =>
		FileSystem{
		    mounts,
		    root
		},
	    _ => bail!("Not a directory")
	};
    fs.save_to_file(out)?;
    Ok(())
}

fn dump(mut pargs:Arguments)->Result<()> {
    let input : OsString = pargs.value_from_str("--in")?;
    let fs = FileSystem::from_file(input)?;
    fs.dump();
    Ok(())
}

struct Finder {
    sd:SigintDetector
}

impl Finder {
    fn do_find_dir(&mut self,
		   fs:&FileSystem,dir:&Directory,re:&Regex,path:&Path,
		   limit:&mut usize)->Result<()> {
	for (name,entry) in dir.entries.iter() {
	    if *limit == 0 {
		return Ok(());
	    }
	    if self.sd.interrupted() {
		bail!("Interrupted");
	    }
	    let mut pb = PathBuf::from(path);
	    pb.push(name);
	    let u = pb.as_os_str().to_string_lossy();
	    if re.is_match(&u) {
		println!("{}",u);
		*limit -= 1;
	    }
	    match entry {
		Entry::Dir(dir) => {
		    let mut pb = PathBuf::from(path);
		    pb.push(name);
		    self.do_find_dir(fs,dir,re,&pb,limit)?;
		},
		_ => ()
	    }
	}
	Ok(())
    }

    fn do_find(&mut self,fs:&FileSystem,pat:&str,limit:&mut usize,case:bool)->Result<()> {
	let re = RegexBuilder::new(pat).case_insensitive(case).build()?;
	let path = Path::new("/");
	self.do_find_dir(fs,&fs.root,&re,&path,limit)?;
	Ok(())
    }

    fn do_find_multi(&mut self,fs:&[(OsString,FileSystem)],pat:&str,
		     limit:&mut usize,case:bool)->Result<()> {
	for (path,fs) in fs.iter() {
	    println!("{:?}:",path);
	    self.do_find(fs,pat,limit,case)?;
	}
	Ok(())
    }

    fn new(sd:SigintDetector)->Self {
	Self { sd }
    }
}

struct ExaminerCli {
    fs:Vec<(OsString,FileSystem)>,
    limit:usize
}

impl ExaminerCli {
    pub fn new(fs:Vec<(OsString,FileSystem)>)->Self {
	Self {
	    fs,
	    limit:1000
	}
    }

    pub fn handle_input(&mut self,u:&str)->Result<bool> {
	let sd = SigintDetector::new();
	let us : Vec<&str> = u.split_ascii_whitespace().collect();
	match &us[..] {
	    [cmd@("find"|"findi"),pat] => {
		let mut limit = self.limit;
		let mut finder = Finder::new(sd);
		finder.do_find_multi(&self.fs,
					  pat,&mut limit,*cmd == "findi")?;
	    },
	    ["limit",l] => self.limit = l.parse()?,
	    ["quit"] => {
		std::process::exit(0);
		// return Ok(true);
	    },
	    [] => (),
	    _ => return Err(anyhow!("Unknown command"))
	}
	Ok(false)
    }
}

fn examine(args:Arguments)->Result<()> {
    info!("Loading inputs");
    let inputs = args.finish();
    let fs : Vec<(OsString,FileSystem)> =
	inputs
	.iter()
	.map(|path| FileSystem::from_file(path).map(|fs| (path.clone(),fs)))
	.flatten()
	.collect();
    let mut cli = ExaminerCli::new(fs);

    let config = rustyline::config::Config::builder()
	.auto_add_history(true)
	.build();
    let mut rl : rustyline::Editor<(),_> = rustyline::Editor::with_config(config)?;
    let _ = rl.load_history(".fsmap-hist");

    loop {
	match rl.readline("> ")
	    .map_err(|e| e.into())
	    .and_then(|u| {
		cli.handle_input(u.as_str())
	    }) {
		Ok(true) => break,
		Ok(false) => (),
		Err(e) => eprintln!("ERR: {}",e)
	}
    }
    Ok(())
}

fn main()->Result<()> {
    let mut args = Arguments::from_env();
    env_logger::Builder::from_env("FSMAP_LOG").init();

    let cmds : &[(&str,Box<dyn Fn(Arguments)->Result<()>>)] = &[
	("collect",Box::new(collect)),
	("dump",Box::new(dump)),
	("examine",Box::new(examine)),
    ];

    match args.subcommand()?
	.and_then(|s| cmds.iter().find(|&(name,_)| name == &s.as_str())) {
	    None => {
		bail!("Specify subcommand (one of {:?})",
		      cmds.iter().map(|&(name,_)| name).collect::<Vec<&str>>()
		)
	    },
	    Some((_,f)) => f(args)
	}
}
