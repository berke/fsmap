use std::cell::Cell;
use std::sync::atomic::{AtomicBool,Ordering};

static mut INTERRUPTED : AtomicBool = AtomicBool::new(false);

extern "C" fn sigint_handler(_sig:std::ffi::c_int) {
    unsafe {
	INTERRUPTED.store(true,Ordering::SeqCst);
    }
}

pub struct SigintDetector {
    old:libc::sigaction,
    interrupted:Cell<bool>
}

impl SigintDetector {
    pub fn new()->Self {
	let old =
	    unsafe {
		let mut sa_mask : libc::sigset_t = std::mem::zeroed();
		libc::sigemptyset(&mut sa_mask);
		let sa_sigaction : libc::size_t = sigint_handler as libc::size_t;

		let act = libc::sigaction {
		    sa_sigaction,
		    sa_mask,
		    sa_flags:0,
		    sa_restorer:None
		};
		let mut old : libc::sigaction = std::mem::zeroed();
		libc::sigaction(
		    libc::SIGINT,
		    &act,
		    &mut old);
		old
	    };
	Self { old,
	       interrupted:Cell::new(false) }
    }

    pub fn interrupted(&self)->bool {
	if let Ok(true) = unsafe { INTERRUPTED.compare_exchange(
	    true,
	    false,
	    Ordering::SeqCst,
	    Ordering::SeqCst)
	} {
	    self.interrupted.set(true);
	    true
	} else {
	    self.interrupted.get()
	}
    }
}

impl Drop for SigintDetector {
    fn drop(&mut self) {
	unsafe {
	    let mut trash : libc::sigaction = std::mem::zeroed();
	    libc::sigaction(
		libc::SIGINT,
		&self.old,
		&mut trash);
	}
    }
}
