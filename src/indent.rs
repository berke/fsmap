pub enum IndentMode {
    None,
    Numbered,
    Spaces
}

impl IndentMode {
    pub fn put_indent(&self,indent:usize) {
	match self {
	    IndentMode::None => (),
	    IndentMode::Numbered => print!(" {:2} ",indent),
	    IndentMode::Spaces => {
		for _ in 0..indent {
		    print!("  ");
		}
	    }
	}
    }
}
