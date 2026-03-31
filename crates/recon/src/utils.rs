/// On windows attach to console if there is one
/// On other platforms do nothing
pub(crate) fn attach() {
    #[cfg(windows)]
    {
        unsafe extern "system" {
            fn AttachConsole(dw_process_id: u32) -> i32;
        }

        const ATTACH_PARENT_PROCESS: u32 = 0xFFFFFFFF;
        // SAFETY: This is safe to call as long as long as we are on Windows.
        unsafe {
            AttachConsole(ATTACH_PARENT_PROCESS);
        }
    }
}
