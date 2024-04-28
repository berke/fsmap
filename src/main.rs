use anyhow::{Result,bail};
use pico_args::Arguments;
use std::ffi::OsString;
use std::path::Path;
use log::{self,info};
use regex::Regex;

mod counter;
mod dumper;
mod examiner_cli;
mod fsexpr;
mod finder;
mod fsmap;
mod scanner;
mod sigint_detector;
mod valve;

use fsexpr::{Expr,Predicate};
use fsmap::*;
use examiner_cli::ExaminerCli;
use counter::Counter;
use dumper::Dumper;
use scanner::Scanner;
use sigint_detector::SigintDetector;

fn collect(mut pargs:Arguments)->Result<()> {
    let path_os : OsString = pargs.value_from_str("--path")?;
    let out : OsString = pargs.value_from_str("--out")?;
    let one_device : bool = pargs.contains("--one-device");
    let mut mounts = Mounts::new();
    let path = Path::new(&path_os);
    let counter = Counter::new();
    let mut scanner = Scanner::new(counter,one_device);
    let fs =
	match scanner.scan(&mut mounts,path)? {
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
    let expr : String = pargs.opt_value_from_str("--pred")?
	.unwrap_or_else(|| "%t".to_string());
    let expr = Expr::parse(&expr)?;
    let fs = FileSystem::from_file(input)?;
    let sd = SigintDetector::new();
    let mut dp = Dumper::new(&sd,&fs,&expr);
    dp.dump()?;
    Ok(())
}

fn examine(args:Arguments)->Result<()> {
    info!("Loading inputs");
    let inputs = args.finish();
    let fs : Vec<(OsString,FileSystem)> =
	inputs
	.iter()
	.flat_map(|path| FileSystem::from_file(path).map(|fs| (path.clone(),fs)))
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

    type Command = Box<dyn Fn(Arguments)->Result<()>>;

    let cmds : &[(&str,Command)] = &[
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
