pub enum Targets {
    Window(Window),
    Monitor(Monitor),
}

#[derive(Debug, thiserror::Error)]
pub enum TargetError {
    #[error("Unable to get windows")]
    UnableToGetWindows(#[from] windows_capture::window::Error),
    #[error("Unable to get monitors")]
    UnableToGetMonitors(#[from] windows_capture::monitor::Error),
}
#[derive(Debug, Clone)]
pub struct Window {
    pub title: String,
    pub monitor: Option<Monitor>,
    pub rect: Rect,
}

#[derive(Debug, Clone)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[derive(Debug, Clone)]
pub struct Monitor {
    pub name: String,
    pub height: u32,
    pub width: u32,
    pub refresh_rate: u32,
    pub index: usize,
}

#[cfg(windows)]
pub(crate) enum NativeTarget {
    Window(windows_capture::window::Window),
    Monitor(windows_capture::monitor::Monitor),
}

#[cfg(windows)]
impl From<windows_capture::window::Window> for Window {
    fn from(window: windows_capture::window::Window) -> Self {
        let rect = window.rect().unwrap();
        Self {
            title: window.title().unwrap(),
            monitor: window.monitor().map(|monitor| monitor.into()),
            rect: Rect {
                left: rect.left,
                top: rect.top,
                right: rect.right,
                bottom: rect.bottom,
            },
        }
    }
}

#[cfg(windows)]
impl From<windows_capture::monitor::Monitor> for Monitor {
    fn from(monitor: windows_capture::monitor::Monitor) -> Self {
        Self {
            name: monitor.name().unwrap(),
            height: monitor.height().unwrap(),
            width: monitor.width().unwrap(),
            refresh_rate: monitor.refresh_rate().unwrap(),
            index: monitor.index().unwrap(),
        }
    }
}

#[cfg(windows)]
impl Targets {
    pub fn get_primary_monitor() -> Result<Monitor, TargetError> {
        Ok(windows_capture::monitor::Monitor::primary()?.into())
    }

    pub fn get_foreground_window() -> Result<Window, TargetError> {
        Ok(windows_capture::window::Window::foreground()?.into())
    }

    pub fn get_windows() -> Result<Vec<Window>, TargetError> {
        Ok(windows_capture::window::Window::enumerate()?
            .into_iter()
            .map(|window| window.into())
            .collect())
    }

    pub fn get_monitors() -> Result<Vec<Monitor>, TargetError> {
        Ok(windows_capture::monitor::Monitor::enumerate()?
            .into_iter()
            .map(|monitor| monitor.into())
            .collect())
    }

    pub(crate) fn to_native(&self) -> Result<NativeTarget, TargetError> {
        match self {
            Self::Window(window) => Ok(NativeTarget::Window(
                windows_capture::window::Window::from_name(&window.title)?,
            )),
            Self::Monitor(monitor) => Ok(NativeTarget::Monitor(
                windows_capture::monitor::Monitor::from_index(monitor.index)?,
            )),
        }
    }
}
