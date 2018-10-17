use SCHEDULER;
use pi::timer;
use traps::TrapFrame;
use process::{Process, State};
use pi::console::kprintln;

/// Sleep for `ms` milliseconds.
///
/// This system call takes one parameter: the number of milliseconds to sleep.
///
/// In addition to the usual status value, this system call returns one
/// parameter: the approximate true elapsed time from when `sleep` was called to
/// when `sleep` returned.
pub fn sleep(ms: u32, tf: &mut TrapFrame) {
    let start_time = timer::current_time();
    let start_time_outside = start_time.clone();

    let poll_fn = Box::new(move |_p: &mut Process| {
        let diff = timer::current_time().wrapping_sub(start_time);
        if diff as u32 > (ms * 1000) {
            return true;
        } else {
            return false;
        }
    });

    SCHEDULER.switch(State::Waiting(poll_fn), tf);
    tf.x7 = 0;
    let diff = (timer::current_time().wrapping_sub(start_time_outside)) / 1000;
    tf.x0 = diff.into();
}

pub fn handle_syscall(num: u16, tf: &mut TrapFrame) {
    match num {
        1 => {
            kprintln!("sleep syscall");
            sleep(tf.x0 as u32, tf);
            tf.elr = tf.elr + 0x04;
        },
        _ => {
            tf.x7 = 1;
            tf.elr = tf.elr + 0x04;
        }
    }
}
