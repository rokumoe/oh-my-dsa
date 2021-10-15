use std::sync::{Mutex, MutexGuard, Once};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct PseudoRand {
    s: u64,
}

impl PseudoRand {
    fn new(seed: u64) -> Self {
        Self { s: seed }
    }

    fn rand(&mut self) -> u64 {
        let x = self.s.wrapping_mul(1103515245).wrapping_add(12345);
        self.s = x;
        x
    }
}

static mut PSEUDO_RAND: Option<Mutex<PseudoRand>> = None;
static mut PSEUDO_RAND_INIT: Once = Once::new();

unsafe fn get_pseudo_rand() -> MutexGuard<'static, PseudoRand> {
    PSEUDO_RAND_INIT.call_once(|| {
        PSEUDO_RAND = Some(Mutex::new(PseudoRand::new(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        )));
    });
    PSEUDO_RAND.as_ref().unwrap().lock().unwrap()
}

pub fn random_u64() -> u64 {
    unsafe { get_pseudo_rand() }.rand()
}
