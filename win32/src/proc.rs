//! Windows process and COM management.

use crate::chk;
use ::std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
use ::windows::Win32::System::{
    Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED},
    Memory::{HeapEnableTerminationOnCorruption, HeapSetInformation},
};

/// Enables the terminate-on-corruption feature. If the heap manager detects an
/// error in any heap used by the process, it calls the Windows Error Reporting
/// service and terminates the process. After a process enables this feature, it
/// cannot be disabled.
///
/// Returns `true` if heap protection was successfully enabled and `false` if
/// the OS version could not support the request.
pub fn enable_heap_protection() -> bool {
    unsafe { HeapSetInformation(None, HeapEnableTerminationOnCorruption, None, 0).as_bool() }
}

thread_local! {
    static COM_LIBRARY_HANDLE: RefCell<Weak<ComLibraryHandle>> = RefCell::new(Weak::new());
}

/// A RAII type which ensures the COM library is loaded for a given thread.
/// Initializes the COM library for use by the calling thread, sets the thread's
/// concurrency model to apartment threading, and creates a new apartment for
/// the thread if one is
///
/// required.  Apartment-threading, while allowing for multiple threads of
/// execution, serializes all incoming calls by requiring that calls to methods
/// of objects created by this thread always run on the same thread, i.e. the
/// apartment/thread that created them. In addition, calls can arrive only at
/// message-queue boundaries.
///
/// This handle must be acquired for every thread that might use COM objects.
///
/// # Usage
///
/// ```rust
/// {
///     let _handle = ComLibraryHandle::load();
///
///     // Do stuff...
///
///     // handle implicitly dropped & unregistered.
/// }
/// ```
pub struct ComLibraryHandle(());

impl ComLibraryHandle {
    /// Acquire a ref-counted handle to the COM library for the calling thread.
    ///
    /// This should ideally be called only once on thread creation on dropped on
    /// thread termination, but repeated calls to [`acquire`](Self::acquire)
    /// will not cause problems due to the ref-counted return type provided all
    /// returned values are dropped appropriately when the thread exits.
    pub fn acquire() -> Rc<Self> {
        COM_LIBRARY_HANDLE.with(|cell| {
            let cell_ref = cell.borrow();
            if let Some(h) = Weak::upgrade(&*cell_ref) {
                h
            } else {
                drop(cell_ref);
                ::tracing::debug!("Initializing COM library (apartment-threaded)");
                chk!(res; CoInitializeEx(None, COINIT_APARTMENTTHREADED )).unwrap();
                let handle = Rc::new(Self(()));
                cell.replace(Rc::downgrade(&handle));
                handle
            }
        })
    }
}

impl Drop for ComLibraryHandle {
    fn drop(&mut self) {
        ::tracing::debug!("Uninitializing COM library");
        unsafe {
            CoUninitialize();
        }
    }
}
