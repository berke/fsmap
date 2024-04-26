use std::time::Instant;

pub struct Valve {
    pub mask:u64,
    last:Instant,
    threshold:f64
}

impl Valve {
    pub fn new(threshold:f64)->Self {
	Self{
	    mask:1,
	    last:Instant::now(),
	    threshold
	}
    }

    pub fn tick(&mut self) {
	let now = Instant::now();
	let dur = now.duration_since(self.last);
	let dt = dur.as_secs_f64();
	if dt > 2.0 * self.threshold {
	    self.mask >>= 1;
	} else if dt < self.threshold / 2.0 {
	    self.mask = self.mask.wrapping_shl(1) | 1;
	}
	self.last = now;
    }
}
