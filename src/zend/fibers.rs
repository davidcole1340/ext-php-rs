
use crate::boxed::ZBox;
use crate::prelude::PhpResult;
use crate::types::ZendHashTable;
use crate::zend::Function;

use std::cell::RefCell;
use std::fs::File;
use std::io;
use std::os::fd::{RawFd, FromRawFd};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::io::Read;
use lazy_static::lazy_static;
use tokio::runtime::Runtime;
use std::os::fd::AsRawFd;

lazy_static! {
    pub static ref RUNTIME: Runtime = Runtime::new().expect("Could not allocate runtime");
}

#[cfg(any(target_os = "linux", target_os = "solaris"))]
fn sys_pipe() -> io::Result<(RawFd, RawFd)> {
    let mut pipefd = [0; 2];
    let ret = unsafe { libc::pipe2(pipefd.as_mut_ptr(), libc::O_CLOEXEC | libc::O_NONBLOCK) };
    if ret == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok((pipefd[0], pipefd[1]))
}

pub struct EventLoop {
    fibers: ZBox<ZendHashTable>,

    sender: Sender<u64>,
    receiver: Receiver<u64>,

    notify_sender: File,
    notify_receiver: File,

    get_current_suspension: Function,

    dummy: [u8; 1],
}

impl EventLoop {
    pub fn new() -> PhpResult<Self> {
        let (sender, receiver) = channel();
        let (notify_receiver, notify_sender) =
            sys_pipe().map_err(|err| format!("Could not create pipe: {}", err))?;

        call_user_func!(Function::from_function("class_exists"), "\\Revolt\\EventLoop")?;
        call_user_func!(Function::from_function("interface_exists"), "\\Revolt\\EventLoop\\Suspension")?;

        Ok(Self {
            fibers: ZendHashTable::new(),
            sender: sender,
            receiver: receiver,
            notify_sender: unsafe { File::from_raw_fd(notify_sender) },
            notify_receiver: unsafe { File::from_raw_fd(notify_receiver) },
            dummy: [0; 1],
            get_current_suspension: Function::try_from_method("\\Revolt\\EventLoop", "getSuspension").ok_or("\\Revolt\\EventLoop::getSuspension does not exist")?,
        })
    }

    pub fn get_event_fd(&self)->u64 {
        self.notify_receiver.as_raw_fd() as u64
    }
    
    pub fn wakeup_internal(&mut self) -> PhpResult<()> {
        self.notify_receiver.read_exact(&mut self.dummy).unwrap();

        for fiber_id in self.receiver.try_iter() {
            if let Some(fiber) = self.fibers.get_index_mut(fiber_id) {
                fiber.object_mut().unwrap().try_call_method("resume", vec![])?;
                self.fibers.remove_index(fiber_id);
            }
        }
        Ok(())
    }

    pub fn prepare_resume(&mut self) -> u64 {
        let idx = self.fibers.len() as u64;
        self.fibers.insert_at_index(idx, call_user_func!(self.get_current_suspension).unwrap()).unwrap();
        
        idx
    }

    pub fn suspend() {
        EVENTLOOP.with_borrow_mut(|c| {
            let c = c.as_mut().unwrap();
            call_user_func!(c.get_current_suspension).unwrap().try_call_method("suspend", vec![]).unwrap();
        });
    }

    pub fn get_sender(&self) -> Sender<u64> {
        self.sender.clone()
    }
    pub fn get_notify_sender(&self) -> File {
        self.notify_sender.try_clone().unwrap()
    }
}

thread_local! {
    pub static EVENTLOOP: RefCell<Option<EventLoop>> = RefCell::new(None);
}
