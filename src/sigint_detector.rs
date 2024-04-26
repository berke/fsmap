use std::sync::atomic::{AtomicBool,Ordering};

static mut INTERRUPTED : AtomicBool = AtomicBool::new(false);

extern "C" fn sigint_handler(_sig:std::ffi::c_int) {
    unsafe {
	INTERRUPTED.store(true,Ordering::SeqCst);
    }
}

type SignalHandler = extern "C" fn(libc::c_int);

pub struct SigintDetector {
    old:libc::sigaction
}

impl SigintDetector {
    pub fn new()->Self {
	let old =
	    unsafe {
		let mut sa_mask : libc::sigset_t = std::mem::zeroed();
		libc::sigemptyset(&mut sa_mask);
		let sa_sigaction : libc::size_t =
		    std::mem::transmute::<SignalHandler,usize>(sigint_handler);

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
	Self { old }
    }

    pub fn interrupted(&self)->bool {
	unsafe { INTERRUPTED.compare_exchange(
	    true,
	    false,
	    Ordering::SeqCst,
	    Ordering::SeqCst)
	}.unwrap_or(false)
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
