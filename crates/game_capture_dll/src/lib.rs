//! # hudhook

#![allow(clippy::needless_doctest_main)]
#![allow(static_mut_refs)]
#![deny(missing_docs)]

use std::cell::OnceCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use tracing::{debug, error, trace};
use windows::core::Error;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, WPARAM};
use windows::Win32::System::Console::{
    AllocConsole, FreeConsole, GetConsoleMode, GetStdHandle, SetConsoleMode, CONSOLE_MODE,
    ENABLE_VIRTUAL_TERMINAL_PROCESSING, STD_OUTPUT_HANDLE,
};
use windows::Win32::System::LibraryLoader::FreeLibraryAndExitThread;
pub use {tracing, windows};

use crate::mh::{MH_ApplyQueued, MH_Initialize, MH_Uninitialize, MhHook, MH_STATUS};

pub mod hooks;
#[cfg(feature = "inject")]
pub mod inject;
pub mod mh;
pub(crate) mod renderer;

pub use renderer::msg_filter::MessageFilter;

pub mod util;

// Global state objects.
static mut MODULE: OnceCell<HINSTANCE> = OnceCell::new();
static mut HUDHOOK: OnceCell<Hudhook> = OnceCell::new();
static CONSOLE_ALLOCATED: AtomicBool = AtomicBool::new(false);

/// Allocate a Windows console.
pub fn alloc_console() -> Result<(), Error> {
    if !CONSOLE_ALLOCATED.swap(true, Ordering::SeqCst) {
        unsafe { AllocConsole()? };
    }

    Ok(())
}

/// Enable console colors if the console is allocated.
pub fn enable_console_colors() {
    if CONSOLE_ALLOCATED.load(Ordering::SeqCst) {
        unsafe {
            // Get the stdout handle
            let stdout_handle = GetStdHandle(STD_OUTPUT_HANDLE).unwrap();

            // Call GetConsoleMode to get the current mode of the console
            let mut current_console_mode = CONSOLE_MODE(0);
            GetConsoleMode(stdout_handle, &mut current_console_mode).unwrap();

            // Set the new mode to include ENABLE_VIRTUAL_TERMINAL_PROCESSING for ANSI
            // escape sequences
            current_console_mode.0 |= ENABLE_VIRTUAL_TERMINAL_PROCESSING.0;

            // Call SetConsoleMode to set the new mode
            SetConsoleMode(stdout_handle, current_console_mode).unwrap();
        }
    }
}

/// Free the previously allocated Windows console.
pub fn free_console() -> Result<(), Error> {
    if CONSOLE_ALLOCATED.swap(false, Ordering::SeqCst) {
        unsafe { FreeConsole()? };
    }

    Ok(())
}

/// Disable hooks and eject the DLL.
///
/// ## Ejecting a DLL
///
/// To eject your DLL, invoke the [`eject`] method from anywhere in your
/// render loop. This will disable the hooks, free the console (if it has
/// been created before) and invoke
/// [`windows::Win32::System::LibraryLoader::FreeLibraryAndExitThread`].
///
/// Befor calling [`eject`], make sure to perform any manual cleanup (e.g.
/// dropping/resetting the contents of static mutable variables).
pub fn eject() {
    thread::spawn(|| unsafe {
        if let Err(e) = free_console() {
            error!("{e:?}");
        }

        if let Some(mut hudhook) = HUDHOOK.take() {
            if let Err(e) = hudhook.unapply() {
                error!("Couldn't unapply hooks: {e:?}");
            }
        }

        if let Some(module) = MODULE.take() {
            FreeLibraryAndExitThread(module, 0);
        }
    });
}

/// Generic trait for platform-specific hooks.
///
/// Implement this if you are building a custom hook for a non-supported
/// renderer.
///
/// Check out first party implementations for guidance on how to implement the
/// methods:
/// - [`ImguiDx9Hooks`](crate::hooks::dx9::ImguiDx9Hooks)
/// - [`ImguiDx11Hooks`](crate::hooks::dx11::ImguiDx11Hooks)
/// - [`ImguiDx12Hooks`](crate::hooks::dx12::ImguiDx12Hooks)
/// - [`ImguiOpenGl3Hooks`](crate::hooks::opengl3::ImguiOpenGl3Hooks)
pub trait Hooks {
    /// Construct a boxed instance of the implementor, storing the provided
    /// render loop where appropriate.
    fn from_render_loop<T>(t: T) -> Box<Self>
    where
        Self: Sized,
        T: Send + Sync + 'static;

    /// Return the list of hooks to be enabled, in order.
    fn hooks(&self) -> &[MhHook];

    /// Cleanup global data and disable the hooks.
    ///
    /// # Safety
    ///
    /// Is most definitely UB.
    unsafe fn unhook(&mut self);
}

/// Holds all the activated hooks and manages their lifetime.
pub struct Hudhook(Vec<Box<dyn Hooks>>);
unsafe impl Send for Hudhook {}
unsafe impl Sync for Hudhook {}

impl Hudhook {
    /// Create a builder object.
    pub fn builder() -> HudhookBuilder {
        HudhookBuilder(Hudhook::new())
    }

    fn new() -> Self {
        // Initialize minhook.
        match unsafe { MH_Initialize() } {
            MH_STATUS::MH_ERROR_ALREADY_INITIALIZED | MH_STATUS::MH_OK => {}
            status @ MH_STATUS::MH_ERROR_MEMORY_ALLOC => panic!("MH_Initialize: {status:?}"),
            _ => unreachable!(),
        }

        Hudhook(Vec::new())
    }

    /// Return an iterator of all the activated raw hooks.
    fn hooks(&self) -> impl IntoIterator<Item = &MhHook> {
        self.0.iter().flat_map(|h| h.hooks())
    }

    /// Apply the hooks.
    pub fn apply(self) -> Result<(), MH_STATUS> {
        // Queue enabling all the hooks.
        for hook in self.hooks() {
            unsafe { hook.queue_enable()? };
        }

        // Apply the queue of enable actions.
        unsafe { MH_ApplyQueued().ok_context("MH_ApplyQueued")? };

        unsafe { HUDHOOK.set(self).ok() };

        Ok(())
    }

    /// Disable and cleanup the hooks.
    pub fn unapply(&mut self) -> Result<(), MH_STATUS> {
        // Queue disabling all the hooks.
        for hook in self.hooks() {
            unsafe { hook.queue_disable()? };
        }

        // Apply the queue of disable actions.
        unsafe { MH_ApplyQueued().ok_context("MH_ApplyQueued")? };

        // Uninitialize minhook.
        unsafe { MH_Uninitialize().ok_context("MH_Uninitialize")? };

        // Invoke cleanup for all hooks.
        for hook in &mut self.0 {
            unsafe { hook.unhook() };
        }

        Ok(())
    }
}

/// Builder object for [`Hudhook`].
///
/// Example usage:
/// ```no_run
/// use hudhook::hooks::dx12::ImguiDx12Hooks;
/// use hudhook::hooks::ImguiRenderLoop;
/// use hudhook::*;
///
/// pub struct MyRenderLoop;
///
/// impl ImguiRenderLoop for MyRenderLoop {
///     fn render(&mut self, frame: &mut imgui::Ui) {
///         // ...
///     }
/// }
///
/// #[no_mangle]
/// pub unsafe extern "stdcall" fn DllMain(
///     hmodule: HINSTANCE,
///     reason: u32,
///     _: *mut std::ffi::c_void,
/// ) {
///     if reason == DLL_PROCESS_ATTACH {
///         std::thread::spawn(move || {
///             let hooks = Hudhook::builder()
///                 .with::<ImguiDx12Hooks>(MyRenderLoop())
///                 .with_hmodule(hmodule)
///                 .build();
///             hooks.apply();
///         });
///     }
/// }
pub struct HudhookBuilder(Hudhook);

impl HudhookBuilder {
    /// Save the DLL instance (for the [`eject`] method).
    pub fn with_hmodule(self, module: HINSTANCE) -> Self {
        unsafe { MODULE.set(module).unwrap() };
        self
    }

    /// Build the [`Hudhook`] object.
    pub fn build(self) -> Hudhook {
        self.0
    }
}

#[no_mangle]
/// Entry point for the DLL.
pub unsafe extern "stdcall" fn DllMain(
    hmodule: HINSTANCE,
    reason: u32,
    _: *mut ::std::ffi::c_void,
) {
    match reason {
        ::windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH => {
            debug!("DLL_PROCESS_ATTACH");
            if let Err(e) = Hudhook::builder().with_hmodule(hmodule).build().apply() {
                error!("Couldn't apply hooks: {e:?}");
                eject();
            }
        }
        ::windows::Win32::System::SystemServices::DLL_PROCESS_DETACH => {
            debug!("DLL_PROCESS_DETACH");
        }
        _ => {
            debug!("DLL reason: {reason}");
        }
    }
}
