//! Windows process management features.

use ::windows::Win32::System::Memory::{HeapEnableTerminationOnCorruption, HeapSetInformation};

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
