use std::ffi::{c_void, CStr, OsStr};
use std::fmt;

use dandelion_agent_lib::anyhow::{anyhow, Result};
use dandelion_agent_lib::Agent;
use dlopen::symbor::Container;

pub struct ExternalCollection {
    sym: Container<abi1::Symbols<'static>>,
}

impl ExternalCollection {
    pub fn open<S: AsRef<OsStr>>(name: S) -> Result<Self> {
        let sym = unsafe { Container::load(name)? };
        Ok(Self { sym })
    }

    pub fn len(&self) -> usize {
        (self.sym.count)()
    }

    pub fn name(&self, index: usize) -> &CStr {
        let ptr = (self.sym.name)(index);
        // SAFETY: the returned pointer has the same lifetime as self.
        unsafe { CStr::from_ptr(ptr) }
    }

    pub fn description(&self, index: usize) -> &CStr {
        let ptr = (self.sym.desc)(index);
        // SAFETY: the returned pointer has the same lifetime as self.
        unsafe { CStr::from_ptr(ptr) }
    }

    pub(crate) fn alloc<'coll>(
        &'coll self,
        index: usize,
        args: &[&str],
    ) -> Result<Box<dyn Agent + 'coll>> {
        let sym = &self.sym;
        let encoded = external_args(args)?;
        let args = encoded.as_ptr();
        let mut errptr = std::ptr::null();
        let pointer = unsafe {
            // SAFETY: args is borrowed for this call.  On success, the returned pointer must be
            // freed by inst_free.  On failure, errptr is populated with an error string which must
            // be freed with err_free.
            (sym.inst_alloc)(index, args.cast(), &mut errptr)
        };
        if pointer.is_null() {
            Err(sym.error_from_errptr(errptr, "alloc"))
        } else {
            Ok(Box::new(DynamicAgent { sym, pointer }))
        }
    }
}

impl fmt::Debug for ExternalCollection {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str("ExternalCollection { ... }")
    }
}

struct DynamicAgent<'coll> {
    sym: &'coll Container<abi1::Symbols<'static>>,
    pointer: *mut c_void,
}

impl<'coll> Drop for DynamicAgent<'coll> {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: pointer was returned by inst_alloc, and must be freed with inst_free.
            (self.sym.inst_free)(self.pointer);
        }
    }
}

impl<'coll> Agent for DynamicAgent<'coll> {
    fn init(&mut self, _args: &[&str]) -> Result<()> {
        unreachable!()
    }

    fn send(&mut self, data: &[u8]) -> Result<()> {
        let mut errptr = std::ptr::null();
        let ok = unsafe {
            // SAFETY: pointer was returned by inst_alloc; data is borrowed for this call.  On
            // failure, errptr is populated with an error string which must be freed with err_free.
            (self.sym.send)(self.pointer, data.as_ptr(), data.len(), &mut errptr)
        };
        if ok {
            Ok(())
        } else {
            Err(self.sym.error_from_errptr(errptr, "send"))
        }
    }

    fn recv_begin(&mut self) -> Result<usize> {
        let mut count = 0;
        let mut errptr = std::ptr::null();
        let ok = unsafe {
            // SAFETY: pointer was returned by inst_alloc.  On success, count is populated.  On
            // failure, errptr is populated with an error string which must be freed with err_free.
            (self.sym.recv_begin)(self.pointer, &mut count, &mut errptr)
        };
        if ok {
            Ok(count)
        } else {
            Err(self.sym.error_from_errptr(errptr, "recv_begin"))
        }
    }

    fn recv_read(&mut self, index: usize, into: &mut Vec<u8>) -> Result<()> {
        let mut len = 0;
        let mut errptr = std::ptr::null();
        let data = unsafe {
            // SAFETY: pointer was returned by inst_alloc.  On success, a borrowed pointer is
            // returned and len is populated with its length.  On failure, errptr is populated with
            // an error string which must be freed with err_free.
            (self.sym.recv_read)(self.pointer, index, &mut len, &mut errptr)
        };
        if data.is_null() {
            Err(self.sym.error_from_errptr(errptr, "recv_read"))
        } else {
            let data = unsafe {
                // SAFETY: The returned pointer is valid until the next call to inst_recv_* or
                // inst_free.  We're holding a mutex to prevent other threads from doing that.
                std::slice::from_raw_parts(data, len)
            };
            into.extend_from_slice(data);
            Ok(())
        }
    }

    fn recv_commit(&mut self, num: usize) -> Result<()> {
        let mut errptr = std::ptr::null();
        let ok = unsafe {
            // SAFETY: pointer was returned by inst_alloc.  On failure, errptr is populated with an
            // error string which must be freed with err_free.
            (self.sym.recv_commit)(self.pointer, num, &mut errptr)
        };
        if ok {
            Ok(())
        } else {
            Err(self.sym.error_from_errptr(errptr, "recv_commit"))
        }
    }

    fn recv_abort(&mut self) -> Result<()> {
        let mut errptr = std::ptr::null();
        let ok = unsafe {
            // SAFETY: pointer was returned by inst_alloc.  On failure, errptr is populated with an
            // error string which must be freed with err_free.
            (self.sym.recv_abort)(self.pointer, &mut errptr)
        };
        if ok {
            Ok(())
        } else {
            Err(self.sym.error_from_errptr(errptr, "recv_abort"))
        }
    }
}

fn external_args(args: &[&str]) -> Result<Vec<u8>> {
    let mut vec = Vec::new();
    for arg in args {
        if arg.is_empty() {
            return Err(anyhow!("cannot pass empty string as an argument"));
        }
        if arg.contains('\0') {
            return Err(anyhow!("cannot pass a string containing '\\0' as an argument"));
        }
        vec.extend_from_slice(arg.as_bytes());
        vec.push(0);
    }
    vec.push(0);
    Ok(vec)
}

mod abi1 {
    use std::ffi::{c_char, c_void, CStr};

    use dlopen::symbor::{SymBorApi, Symbol};

    use crate::anyhow::{anyhow, Error};

    pub type CountFn = extern "system" fn() -> usize;
    pub type StringFn = extern "system" fn(usize) -> *const c_char;
    pub type ErrFreeFn = unsafe extern "system" fn(*const c_char);
    pub type AllocFn =
        unsafe extern "system" fn(usize, *const c_char, *mut *const c_char) -> *mut c_void;
    pub type FreeFn = unsafe extern "system" fn(*mut c_void);
    pub type SendFn =
        unsafe extern "system" fn(*mut c_void, *const u8, usize, *mut *const c_char) -> bool;
    pub type RecvBeginFn =
        unsafe extern "system" fn(*mut c_void, *mut usize, *mut *const c_char) -> bool;
    pub type RecvReadFn =
        unsafe extern "system" fn(*mut c_void, usize, *mut usize, *mut *const c_char) -> *const u8;
    pub type RecvCommitFn =
        unsafe extern "system" fn(*mut c_void, usize, *mut *const c_char) -> bool;
    pub type RecvAbortFn = unsafe extern "system" fn(*mut c_void, *mut *const c_char) -> bool;

    #[derive(SymBorApi)]
    pub struct Symbols<'a> {
        #[dlopen_name = "dandelion_agent_abi1_count"]
        pub count: Symbol<'a, CountFn>,
        #[dlopen_name = "dandelion_agent_abi1_name"]
        pub name: Symbol<'a, StringFn>,
        #[dlopen_name = "dandelion_agent_abi1_description"]
        pub desc: Symbol<'a, StringFn>,
        #[dlopen_name = "dandelion_agent_abi1_error_free"]
        pub err_free: Symbol<'a, ErrFreeFn>,
        #[dlopen_name = "dandelion_agent_abi1_instance_alloc"]
        pub inst_alloc: Symbol<'a, AllocFn>,
        #[dlopen_name = "dandelion_agent_abi1_instance_free"]
        pub inst_free: Symbol<'a, FreeFn>,
        #[dlopen_name = "dandelion_agent_abi1_instance_send"]
        pub send: Symbol<'a, SendFn>,
        #[dlopen_name = "dandelion_agent_abi1_instance_recv_begin"]
        pub recv_begin: Symbol<'a, RecvBeginFn>,
        #[dlopen_name = "dandelion_agent_abi1_instance_recv_read"]
        pub recv_read: Symbol<'a, RecvReadFn>,
        #[dlopen_name = "dandelion_agent_abi1_instance_recv_commit"]
        pub recv_commit: Symbol<'a, RecvCommitFn>,
        #[dlopen_name = "dandelion_agent_abi1_instance_recv_abort"]
        pub recv_abort: Symbol<'a, RecvAbortFn>,
    }

    impl Symbols<'_> {
        pub fn error_from_errptr(&self, errptr: *const c_char, operation: &str) -> Error {
            if errptr.is_null() {
                anyhow!("{operation} failed, but did not provide an error message")
            } else {
                let err;
                unsafe {
                    err = anyhow!(CStr::from_ptr(errptr).to_string_lossy());
                    (self.err_free)(errptr);
                }
                err
            }
        }
    }
}
