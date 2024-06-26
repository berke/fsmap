use anyhow::{Result,bail};
use pico_args::Arguments;
use std::ffi::OsString;
use std::path::{Path,PathBuf};
use log::{self,info,warn,LevelFilter};
use rustyline as rl;

mod basic_printer;
mod boolean;
mod counter;
mod dumper;
mod examiner_cli;
mod fsexpr;
mod fsparser;
mod fstok;
mod fsmap;
mod help;
mod indent;
mod limiter;
mod list_printer;
mod scanner;
mod sigint_detector;
mod valve;
mod watcher;

use basic_printer::BasicPrinter;
use fsexpr::FsExpr;
use fsmap::*;
use examiner_cli::ExaminerCli;
use counter::Counter;
use dumper::Dumper;
use scanner::Scanner;
use sigint_detector::SigintDetector;

fn collect(mut args:Arguments)->Result<()> {
    let out : OsString = args.value_from_str("--out")?;
    let one_device : bool = args.contains("--one-device");
    let paths : Vec<OsString> = args.finish();
    if paths.len() != 1 {
	bail!("Exactly one path must be given to collect");
    }
    let path = Path::new(&paths[0]);
    let mut mounts = Mounts::new();
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

fn help(_args:Arguments)->Result<()> {
    print!("{}",help::COMMAND_TEXT);
    Ok(())
}

fn help_expr(_args:Arguments)->Result<()> {
    print!("{}",help::EXPR_TEXT);
    Ok(())
}

fn dump(mut args:Arguments)->Result<()> {
    let expr : String = args.opt_value_from_str("--expr")?
	.unwrap_or_else(|| "%t".to_string());
    let expr = FsExpr::parse(&expr)?;
    let inputs = args.finish();
    let (fss,errs) = FileSystems::load_multiple(&inputs[..]);
    for (path,err) in &errs {
	warn!("Error loading {:?}: {}",path,err);
    }
    let sd = SigintDetector::new();
    let bp = BasicPrinter::new();
    let mut dp = Dumper::new(&sd,&fss,&expr,bp);
    dp.dump()?;
    Ok(())
}

fn examine(mut args:Arguments)->Result<()> {
    info!("Loading inputs");
    let enable_history = !args.contains("--no-history");
    let inputs = args.finish();
    let (fss,errs) = FileSystems::load_multiple(&inputs[..]);
    for (path,err) in &errs {
	warn!("Error loading {:?}: {}",path,err);
    }
    let mut cli = ExaminerCli::new(fss);

    let config = rl::config::Config::builder()
	.auto_add_history(true)
	.build();
    let mut rl : rl::Editor<(),rl::history::FileHistory> =
	rl::Editor::with_config(config)?;
    let mut hist_path = PathBuf::new();
    if let Some(home) = std::env::var_os("HOME") {
	hist_path.push(home);
    }
    hist_path.push(".fsmap-hist");
    if enable_history {
	let _ = rl.load_history(&hist_path);
    }

    loop {
	match rl.readline("> ") {
	    Err(rl::error::ReadlineError::Eof) => break,
	    Err(rl::error::ReadlineError::Interrupted) => println!("^C"),
	    Err(e) => eprintln!("Error: {}",e),
	    Ok(u) => match cli.handle_input(u.as_str()) {
		Ok(true) => break,
		Ok(false) => (),
		Err(e) => eprintln!("Error: {}",e)
	    }
	}
    }

    if enable_history {
	let _ = rl.append_history(&hist_path);
    }
    std::process::exit(0)
}

fn main()->Result<()> {
    let mut args = Arguments::from_env();
    env_logger::Builder::new()
	.filter_level(LevelFilter::Info)
	.parse_env("FSMAP_LOG")
	.init();

    type Command = Box<dyn Fn(Arguments)->Result<()>>;

    let cmds : &[(&str,Command)] = &[
	("collect",Box::new(collect)),
	("dump",Box::new(dump)),
	("examine",Box::new(examine)),
	("help",Box::new(help)),
	("help-expr",Box::new(help_expr))
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
