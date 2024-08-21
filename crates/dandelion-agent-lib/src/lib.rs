pub extern crate anyhow;
pub extern crate linkme;

use std::ffi::CStr;

use anyhow::Result;
use linkme::distributed_slice;

#[distributed_slice]
pub static FACTORY: [Factory];

#[macro_export]
macro_rules! agent_factory {
    ( $const:ident, $ctor:expr, $name:expr, $desc:expr ) => {
        #[::dandelion_agent_lib::linkme::distributed_slice(::dandelion_agent_lib::FACTORY)]
        #[linkme(crate = ::dandelion_agent_lib::linkme)]
        static $const: ::dandelion_agent_lib::Factory =
            ::dandelion_agent_lib::Factory::new($ctor, $name, $desc);
    };
    ( $const:ident, $ctor:expr, $name:expr ) => {
        #[::dandelion_agent_lib::linkme::distributed_slice(::dandelion_agent_lib::FACTORY)]
        #[linkme(crate = ::dandelion_agent_lib::linkme)]
        static $const: ::dandelion_agent_lib::Factory =
            ::dandelion_agent_lib::Factory::new($ctor, $name, c"");
    };
}

#[derive(Debug, Clone, Copy)]
pub struct Factory {
    pub ctor: Constructor,
    pub name: &'static CStr,
    pub desc: &'static CStr,
}

impl Factory {
    pub const fn new(ctor: Constructor, name: &'static CStr, desc: &'static CStr) -> Self {
        Self { ctor, name, desc }
    }
}

pub type Constructor = fn() -> AgentBox;

pub type AgentBox = Box<dyn Agent>;

pub trait Agent {
    fn init(&mut self, args: &[&str]) -> Result<()>;
    fn send(&mut self, data: &[u8]) -> Result<()>;
    fn recv_begin(&mut self) -> Result<usize>;
    fn recv_read(&mut self, index: usize, into: &mut Vec<u8>) -> Result<()>;
    fn recv_commit(&mut self, count: usize) -> Result<()>;

    fn recv_abort(&mut self) -> Result<()> {
        self.recv_commit(0)
    }

    fn poll(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(feature = "export")]
pub mod export {
    use std::ffi::{c_char, c_void, CStr, CString};
    use std::sync::Mutex;
    use std::{fmt, ptr, slice};

    use super::anyhow::{Error, Result};
    use super::{AgentBox, FACTORY};

    struct Instance {
        agent: AgentBox,
        vec: Vec<u8>,
        recv_count: Option<usize>,
    }

    impl Instance {
        fn new(agent: AgentBox) -> Self {
            let vec = Vec::new();
            let recv_count = None;
            Self { agent, vec, recv_count }
        }
    }

    type InstanceMutex = Mutex<Instance>;
    type InstanceBox = Box<InstanceMutex>;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Bug {
        ArgsNotUTF8,
        NotInTxn,
        AlreadyInTxn,
        ReadRange(usize, usize),
        CommitRange(usize, usize),
    }

    impl fmt::Display for Bug {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::ArgsNotUTF8 => write!(fmt, "BUG: all arguments must be valid UTF-8"),
                Self::NotInTxn => write!(fmt, "BUG: not in a recv transaction"),
                Self::AlreadyInTxn => write!(fmt, "BUG: recv transaction already in progress"),
                Self::ReadRange(index, count) => write!(fmt, "BUG: message index {index} out of range, there are only {count} in this recv transaction"),
                Self::CommitRange(limit, count) => write!(fmt, "BUG: message limit {limit} out of range, there are only {count} in this recv transaction"),
            }
        }
    }

    impl std::error::Error for Bug {}

    fn set_error(out: *mut *const c_char, operation: &'static str, err: impl Into<Error>) {
        let err = err.into().context(operation).to_string();

        if out.is_null() {
            eprintln!("error: {err}");
            return;
        }

        let mut vec = Vec::with_capacity(err.len() + 1);
        vec.extend_from_slice(err.as_bytes());
        vec.push(0);
        let cstring = CString::from_vec_with_nul(vec).unwrap();
        unsafe {
            *out = cstring.into_raw();
        }
    }

    fn set_error_and_return<T>(
        retval: T,
        out: *mut *const c_char,
        operation: &'static str,
        err: impl Into<Error>,
    ) -> T {
        set_error(out, operation, err);
        retval
    }

    unsafe fn parse_arg<'a>(ptr: *const u8) -> (&'a CStr, *const u8) {
        unsafe {
            let mut len = 0usize;
            while *ptr.add(len) != 0 {
                len += 1;
            }
            let cstr = CStr::from_bytes_with_nul_unchecked(slice::from_raw_parts(ptr, len + 1));
            let ptr = ptr.add(len + 1);
            (cstr, ptr)
        }
    }

    unsafe fn parse_args<'a>(mut ptr: *const u8) -> Result<Vec<&'a str>> {
        let mut args = Vec::<&'a str>::new();
        let (mut cstr, mut rest) = unsafe { parse_arg::<'a>(ptr) };
        while !cstr.is_empty() {
            args.push(cstr.to_str()?);
            ptr = rest;
            (cstr, rest) = unsafe { parse_arg::<'a>(ptr) };
        }
        Ok(args)
    }

    /// Returns the number of agent implementations available in this dynamic library.
    ///
    #[export_name = "dandelion_agent_abi1_count"]
    pub extern "system" fn count() -> usize {
        FACTORY.len()
    }

    /// Returns the name of an agent implementation.  The returned string is an immutable
    /// '\0'-terminated UTF-8 string with static lifetime.
    ///
    /// `index` identifies the agent implementation, and it must be strictly less than `count()`.
    ///
    #[export_name = "dandelion_agent_abi1_name"]
    pub extern "system" fn name(index: usize) -> *const c_char {
        assert!(index < FACTORY.len());
        FACTORY[index].name.as_ptr()
    }

    /// Returns the textual description of an agent implementation.  The returned string is an
    /// immutable '\0'-terminated UTF-8 string with static lifetime.
    ///
    /// `index` identifies the agent implementation, and it must be strictly less than `count()`.
    ///
    #[export_name = "dandelion_agent_abi1_description"]
    pub extern "system" fn description(index: usize) -> *const c_char {
        assert!(index < FACTORY.len());
        FACTORY[index].desc.as_ptr()
    }

    /// Frees an error string previously returned by a failing ABI call.
    ///
    #[export_name = "dandelion_agent_abi1_error_free"]
    pub unsafe extern "system" fn error_free(err: *const c_char) {
        // SAFETY: err was previously created in set_error, so it was allocated as a CString and
        // leaked using into_raw.
        //
        let err = unsafe { CString::from_raw(err.cast_mut()) };

        // We could just let it go out of scope, but let's be explicit about it for clarity.
        drop(err);
    }

    /// Allocates a new instance of an agent implementation, and initializes that instance with the
    /// provided arguments.
    ///
    /// `index` identifies the agent implementation, and it must be strictly less than `count()`.
    ///
    /// `args` must point to a contiguous sequence of '\0'-terminated UTF-8 strings ending with an
    /// empty string.  This is the same in-memory format used for environment variables.  It must
    /// remain alive and immutable for the duration of the call.
    ///
    /// On success, returns a non-null opaque pointer to the newly allocated instance.  This
    /// pointer must later be freed using instance_free.
    ///
    /// On failure, returns null and populates a non-null `errptr` with an immutable
    /// '\0'-terminated UTF-8 error string.  This error string must later be freed using
    /// error_free.
    ///
    #[export_name = "dandelion_agent_abi1_instance_alloc"]
    pub unsafe extern "system" fn instance_alloc(
        index: usize,
        args: *const c_char,
        errptr: *mut *const c_char,
    ) -> *mut c_void {
        assert!(index < FACTORY.len());
        const OP: &str = "instance_alloc";

        // SAFETY: args is borrowed for the duration of this call.
        let args = unsafe { parse_args(args.cast::<u8>()) };
        let args = match args {
            Ok(x) => x,
            Err(_) => return set_error_and_return(ptr::null_mut(), errptr, OP, Bug::ArgsNotUTF8),
        };
        let mut agent = (FACTORY[index].ctor)();
        if let Err(err) = agent.init(&args) {
            return set_error_and_return(ptr::null_mut(), errptr, OP, err);
        }
        Box::into_raw(Box::new(Mutex::new(Instance::new(agent)))).cast()
    }

    /// Frees an agent previously returned by instance_alloc.
    ///
    #[export_name = "dandelion_agent_abi1_instance_free"]
    pub unsafe extern "system" fn instance_free(mutex: *mut c_void) {
        // SAFETY: mutex was previously returned by instance_alloc, so it was allocated using
        // Box::new and then leaked via Box::into_raw.
        //
        let mutex = unsafe { InstanceBox::from_raw(mutex.cast::<InstanceMutex>()) };

        // We could just let it go out of scope, but let's be explicit about it for clarity.
        drop(mutex);
    }

    /// Sends an outgoing message using an agent previously returned by instance_alloc.  This
    /// enqueues the message for sending, but does not provide any specific guarantees on message
    /// delivery or receipt acknowledgement.
    ///
    /// `ptr` must point to the message, which is a buffer of `len` bytes that must remain alive
    /// and immutable for the duration of the call.
    ///
    /// On failure, populates a non-null `errptr` with an immutable '\0'-terminated UTF-8 error
    /// string.  This error string must later be freed using error_free.
    ///
    #[export_name = "dandelion_agent_abi1_instance_send"]
    extern "system" fn instance_send(
        mutex: *mut c_void,
        ptr: *const u8,
        len: usize,
        errptr: *mut *const c_char,
    ) -> bool {
        const OP: &str = "instance_send";

        // SAFETY: mutex was previously returned by instance_alloc.
        let mutex = unsafe { &*mutex.cast::<InstanceMutex>() };
        // SAFETY: ptr is borrowed for the duration of the call, and points to len bytes.
        let data = unsafe { std::slice::from_raw_parts(ptr, len.try_into().unwrap()) };
        let mut guard = mutex.lock().unwrap();
        let instance = &mut *guard;

        if let Err(err) = instance.agent.send(data) {
            return set_error_and_return(false, errptr, OP, err);
        }
        true
    }

    /// Creates a receive transaction using an agent previously returned by instance_alloc.
    ///
    /// On success, populates `countptr` with the number of messages in the transaction.
    ///
    /// On failure, populates a non-null `errptr` with an immutable '\0'-terminated UTF-8 error
    /// string.  This error string must later be freed using error_free.
    ///
    #[export_name = "dandelion_agent_abi1_instance_recv_begin"]
    extern "system" fn instance_recv_begin(
        mutex: *mut c_void,
        countptr: *mut usize,
        errptr: *mut *const c_char,
    ) -> bool {
        const OP: &str = "instance_recv_begin";

        // SAFETY: mutex was previously returned by instance_alloc.
        let mutex = unsafe { &*mutex.cast::<InstanceMutex>() };
        let mut guard = mutex.lock().unwrap();
        let instance = &mut *guard;

        if instance.recv_count.is_some() {
            return set_error_and_return(false, errptr, OP, Bug::AlreadyInTxn);
        }

        let count = match instance.agent.recv_begin() {
            Ok(count) => count,
            Err(err) => return set_error_and_return(false, errptr, OP, err),
        };
        instance.recv_count = Some(count);
        unsafe {
            // SAFETY: countptr is mutably borrowed.
            *countptr = count;
        }
        true
    }

    /// Reads an incoming message within a receive transaction, using an agent previously returned
    /// by instance_alloc.
    ///
    /// `index` identifies the message within the transaction, and must be strictly less the count
    /// provided by instance_recv_begin.
    ///
    /// On success, returns a non-null pointer to the message blob and populates `lenptr` with its
    /// length in bytes.  This pointer will remain valid and immutable until the next call to
    /// instance_recv_read, instance_recv_commit, instance_recv_abort, or instance_free.
    ///
    /// On failure, returns null and populates a non-null `errptr` with an immutable
    /// '\0'-terminated UTF-8 error string.  This error string must later be freed using
    /// error_free.
    ///
    #[export_name = "dandelion_agent_abi1_instance_recv_read"]
    extern "system" fn instance_recv_read(
        mutex: *mut c_void,
        index: usize,
        lenptr: *mut usize,
        errptr: *mut *const c_char,
    ) -> *const u8 {
        const OP: &str = "instance_recv_read";

        // SAFETY: mutex was previously returned by instance_alloc.
        let mutex = unsafe { &*mutex.cast::<InstanceMutex>() };
        let mut guard = mutex.lock().unwrap();
        let instance = &mut *guard;

        if instance.recv_count.is_none() {
            return set_error_and_return(ptr::null(), errptr, OP, Bug::NotInTxn);
        }

        let count = instance.recv_count.unwrap();
        if index >= count {
            return set_error_and_return(ptr::null(), errptr, OP, Bug::ReadRange(index, count));
        }

        let vec = &mut instance.vec;
        vec.clear();
        if let Err(err) = instance.agent.recv_read(index, vec) {
            return set_error_and_return(ptr::null(), errptr, OP, err);
        }

        let len = vec.len();
        unsafe {
            // SAFETY: lenptr is mutably borrowed.  vec must not be modified until the next call to
            // instance_recv_* or instance_free on this instance.
            *lenptr = len;
            vec.as_ptr()
        }
    }

    /// Commits a receive transaction, using an agent previously returned by instance_alloc.
    ///
    /// The first `num` messages, i.e. those with an index strictly less than `num`, are marked as
    /// having been successfully received, guaranteeing that they will not be included in any
    /// future transactions.  How this is accomplished is implementation-dependent: they may be
    /// deleted immediately, or flagged for GC, or they may be archived permanently in some way.
    ///
    /// On failure, populates a non-null `errptr` with an immutable '\0'-terminated UTF-8 error
    /// string.  This error string must later be freed using error_free.
    ///
    #[export_name = "dandelion_agent_abi1_instance_recv_commit"]
    extern "system" fn instance_recv_commit(
        mutex: *mut c_void,
        num: usize,
        errptr: *mut *const c_char,
    ) -> bool {
        const OP: &str = "instance_recv_commit";

        // SAFETY: mutex was previously returned by instance_alloc.
        let mutex = unsafe { &*mutex.cast::<InstanceMutex>() };
        let mut guard = mutex.lock().unwrap();
        let instance = &mut *guard;

        if instance.recv_count.is_none() {
            return set_error_and_return(false, errptr, OP, Bug::NotInTxn);
        }

        let count = instance.recv_count.unwrap();
        if num > count {
            return set_error_and_return(false, errptr, OP, Bug::CommitRange(num, count));
        }

        instance.vec.clear();
        instance.recv_count = None;
        if let Err(err) = instance.agent.recv_commit(num) {
            return set_error_and_return(false, errptr, OP, err);
        }
        true
    }

    /// Aborts a receive transaction, using an agent previously returned by instance_alloc.
    ///
    /// This method is almost-but-not-quite the same as instance_recv_commit with `num` set to 0.
    /// The exception: if called outside of a transaction, this method is a no-op, whereas the
    /// other will fail.
    ///
    /// On failure, populates a non-null `errptr` with an immutable '\0'-terminated UTF-8 error
    /// string.  This error string must later be freed using error_free.
    ///
    #[export_name = "dandelion_agent_abi1_instance_recv_abort"]
    extern "system" fn instance_recv_abort(mutex: *mut c_void, errptr: *mut *const c_char) -> bool {
        const OP: &str = "instance_recv_abort";

        // SAFETY: mutex was previously returned by instance_alloc.
        let mutex = unsafe { &*mutex.cast::<InstanceMutex>() };

        let mut guard = mutex.lock().unwrap();
        let instance = &mut *guard;

        instance.vec.clear();
        instance.recv_count = None;
        if let Err(err) = instance.agent.recv_abort() {
            return set_error_and_return(false, errptr, OP, err);
        }
        true
    }

    /// Polls for asynchronous errors using an agent previously returned by instance_alloc.
    ///
    /// On failure, populates a non-null `errptr` with an immutable '\0'-terminated UTF-8 error
    /// string.  This error string must later be freed using error_free.
    ///
    /// Some agent implementations will treat this as a no-op.
    ///
    /// Some agent implementations will use this to drive a non-blocking asynchronous runtime, and
    /// such agents will want the host to call this method regularly.
    ///
    /// Some agent implementations may launch their own threads, subprocesses, or other concurrent
    /// processing, and such agents will use this to report asynchronous errors.  For these users,
    /// it is recommended to dequeue all pending errors by repeatedly retrying.
    ///
    #[export_name = "dandelion_agent_abi1_instance_poll"]
    extern "system" fn instance_poll(mutex: *mut c_void, errptr: *mut *const c_char) -> bool {
        const OP: &str = "instance_poll";

        // SAFETY: mutex was previously returned by instance_alloc.
        let mutex = unsafe { &*mutex.cast::<InstanceMutex>() };
        let mut guard = mutex.lock().unwrap();
        let instance = &mut *guard;

        if let Err(err) = instance.agent.poll() {
            return set_error_and_return(false, errptr, OP, err);
        }
        true
    }
}
