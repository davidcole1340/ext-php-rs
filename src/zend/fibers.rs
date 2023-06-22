
use crate::boxed::ZBox;
use crate::class::{ClassMetadata, RegisteredClass};
use crate::prelude::PhpResult;
use crate::props::Property;
use crate::types::{ZendHashTable, ZendClassObject};
use crate::zend::Function;

use std::cell::RefCell;
use std::collections::HashMap;
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

pub struct GlobalConnection {
    fibers: ZBox<ZendHashTable>,

    _sender: Sender<u64>,
    receiver: Receiver<u64>,

    _notify_sender: File,
    notify_receiver: File,

    get_current_suspension: Function,
    suspend: Function,
    resume: Function,
    
    dummy: [u8; 1],
}

impl GlobalConnection {
    pub fn getEventFd() -> u64 {
        EVENTLOOP.with_borrow(|c| {
            c.as_ref().unwrap().notify_receiver.as_raw_fd() as u64
        })
    }
    pub fn wakeup() -> PhpResult<()> {
        EVENTLOOP.with_borrow_mut(|c| {
            c.as_mut().unwrap().wakeup_internal()
        })
    }
}

impl GlobalConnection {
    fn new() -> PhpResult<Self> {
        let (sender, receiver) = channel();
        let (notify_receiver, notify_sender) =
            sys_pipe().map_err(|err| format!("Could not create pipe: {}", err))?;

        Ok(Self {
            fibers: ZendHashTable::new(),
            _sender: sender,
            receiver: receiver,
            _notify_sender: unsafe { File::from_raw_fd(notify_sender) },
            notify_receiver: unsafe { File::from_raw_fd(notify_receiver) },
            dummy: [0; 1],
            get_current_suspension: Function::try_from_method("\\Revolt\\EventLoop", "getSuspension").unwrap(),
            suspend: Function::try_from_method("\\Revolt\\EventLoop\\Suspension", "suspend").unwrap(),
            resume: Function::try_from_method("\\Revolt\\EventLoop\\Suspension", "resume").unwrap()
        })
    }
    
    fn wakeup_internal(&mut self) -> PhpResult<()> {
        self.notify_receiver.read_exact(&mut self.dummy).unwrap();

        for fiber_id in self.receiver.try_iter() {
            if let Some(fiber) = self.fibers.get_index_mut(fiber_id) {
                self.resume.try_call_obj(fiber, vec![])?;
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
            let mut suspension = call_user_func!(c.get_current_suspension).unwrap();
            c.suspend.try_call_obj(&mut suspension, vec![])
        }).unwrap();
        ()
    }
}

class_derives!(GlobalConnection);

static EVENTLOOP_META: ClassMetadata<GlobalConnection> = ClassMetadata::new();

impl RegisteredClass for GlobalConnection {
    const CLASS_NAME: &'static str = "GlobalConnection";

    fn get_metadata() -> &'static ClassMetadata<Self> {
        &EVENTLOOP_META
    }

    fn get_properties<'a>() -> HashMap<&'static str, Property<'a, Self>> {
        HashMap::new()
    }
}

thread_local! {
    pub static EVENTLOOP: RefCell<Option<ZBox<ZendClassObject<GlobalConnection>>>> = RefCell::new(None);
}

pub extern "C" fn request_startup(_type: i32, _module_number: i32) -> i32 {
    EVENTLOOP.set(Some(ZendClassObject::new(GlobalConnection::new().unwrap())));
    0
}

pub extern "C" fn request_shutdown(_type: i32, _module_number: i32) -> i32 {
    EVENTLOOP.set(None);
    0
}