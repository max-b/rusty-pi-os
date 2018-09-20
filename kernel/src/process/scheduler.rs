use std::collections::VecDeque;
use std::mem;

use pi::mutex::Mutex;
use pi::console::kprintln;
use pi::interrupt;
use pi::timer;
use aarch64;
use process::{Process, State, Id};
use traps::TrapFrame;
use {start_shell, start_shell_2, print_junk_1};

/// The `tick` time.
// FIXME: When you're ready, change this to something more reasonable.
pub const TICK: u32 = 2 * 1000 * 1000;

/// Process scheduler for the entire machine.
#[derive(Debug)]
pub struct GlobalScheduler(Mutex<Option<Scheduler>>);

impl GlobalScheduler {
    /// Returns an uninitialized wrapper around a local scheduler.
    pub const fn uninitialized() -> GlobalScheduler {
        GlobalScheduler(Mutex::new(None))
    }

    /// Adds a process to the scheduler's queue and returns that process's ID.
    /// For more details, see the documentation on `Scheduler::add()`.
    pub fn add(&self, process: Process) -> Option<Id> {
        self.0.lock().as_mut().expect("scheduler uninitialized").add(process)
    }

    /// Performs a context switch using `tf` by setting the state of the current
    /// process to `new_state`, saving `tf` into the current process, and
    /// restoring the next process's trap frame into `tf`. For more details, see
    /// the documentation on `Scheduler::switch()`.
    #[must_use]
    pub fn switch(&self, new_state: State, tf: &mut TrapFrame) -> Option<Id> {
        self.0.lock().as_mut().expect("scheduler uninitialized").switch(new_state, tf)
    }

    /// Initializes the scheduler and starts executing processes in user space
    /// using timer interrupt based preemptive scheduling. This method should
    /// not return under normal conditions.
    pub fn start(&self) {

        let mut interrupt_controller = interrupt::Controller::new();
        interrupt_controller.enable(interrupt::Interrupt::Timer1);

        timer::tick_in(TICK);

        match Process::new() {
            Some(mut start_process) => {
                let mut new_scheduler = Scheduler::new();

                start_process.trap_frame.elr = start_shell as *const u64 as u64;
                start_process.trap_frame.sp = start_process.stack.top().as_u64();

                // All interrupts unmasked; el0; and aarch64
                start_process.trap_frame.spsr = 0x00;

                let trap_frame_address = (&(*start_process.trap_frame)) as *const TrapFrame as *const u64 as u64;

                kprintln!("start_process = {:#x?}", &start_process);

                new_scheduler.add(start_process);

                let mut process2 = Process::new().unwrap();
                process2.trap_frame.elr = start_shell_2 as *const u64 as u64;
                process2.trap_frame.sp = process2.stack.top().as_u64();

                new_scheduler.add(process2);

                let mut process3 = Process::new().unwrap();
                process3.trap_frame.elr = print_junk_1 as *const u64 as u64;
                process3.trap_frame.sp = process3.stack.top().as_u64();

                new_scheduler.add(process3);

                *self.0.lock() = Some(new_scheduler);

                unsafe {
                    asm!("mov sp, $0"
                         :: "r"(trap_frame_address)
                         :: "volatile");

                    asm!("bl context_restore
                          ldr x1, =_start
                          mov sp, x1
                          mov x1, #0
                          eret" :::: "volatile");
                }

            },
            None => {
                kprintln!("Could not create start process! 🔥🎆🎆🔥");
            }
        }
    }
}

#[derive(Debug)]
struct Scheduler {
    processes: VecDeque<Process>,
    current: Option<Process>,
    last_id: Option<Id>,
}

impl Scheduler {
    /// Returns a new `Scheduler` with an empty queue.
    fn new() -> Scheduler {
        Scheduler {
            processes: VecDeque::new(),
            current: None,
            last_id: None
        }
    }

    /// Adds a process to the scheduler's queue and returns that process's ID if
    /// a new process can be scheduled. The process ID is newly allocated for
    /// the process and saved in its `trap_frame`. If no further processes can
    /// be scheduled, returns `None`.
    ///
    /// If this is the first process added, it is marked as the current process.
    /// It is the caller's responsibility to ensure that the first time `switch`
    /// is called, that process is executing on the CPU.
    fn add(&mut self, mut process: Process) -> Option<Id> {
        let is_first_process = self.last_id.is_none();

        let new_id = match self.last_id {
            Some(id) => id.wrapping_add(1),
            None => 0
        };

        process.trap_frame.tpidr = new_id;
        self.last_id = Some(new_id);

        if is_first_process {
            self.current = Some(process);
        } else {
            self.processes.push_back(process);
        }

        Some(new_id)
    }

    /// Sets the current process's state to `new_state`, finds the next process
    /// to switch to, and performs the context switch on `tf` by saving `tf`
    /// into the current process and restoring the next process's trap frame
    /// into `tf`. If there is no current process, returns `None`. Otherwise,
    /// returns `Some` of the process ID that was context switched into `tf`.
    ///
    /// This method blocks until there is a process to switch to, conserving
    /// energy as much as possible in the interim.
    fn switch(&mut self, new_state: State, tf: &mut TrapFrame) -> Option<Id> {
        let mut current_replacement = None;

        let mut owned_process = mem::replace(&mut self.current, current_replacement);

        if let Some(mut current_process) = owned_process {
            current_process.state = new_state;
            *(current_process.trap_frame) = *tf;
            self.processes.push_back(current_process);
        }

        let ready_index = self.processes.iter_mut().position(|p| p.is_ready());

        match ready_index {
            Some(i) => {
                let mut p = self.processes.remove(i).expect("processes index out of range");
                *tf = *(p.trap_frame);
                p.state = State::Running;

                self.current = Some(p);
                return Some(tf.tpidr);
            },
            None => {
                kprintln!("can't find ready, wfi-ing now...");
                kprintln!("self: {:#x?}", self);

                aarch64::wfi();
                return None;
            }
        }
    }
}
