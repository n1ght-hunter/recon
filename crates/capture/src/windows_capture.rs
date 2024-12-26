use windows_capture::capture::GraphicsCaptureApiHandler;

use crate::target;

struct WindowCaptureHandler {
    control: Control,
}

impl std::fmt::Debug for WindowCaptureHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindowCaptureHandler")
            .field("control", &std::any::type_name::<Control>())
            .finish()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WindowCaptureError {
    #[error(transparent)]
    TargetError(#[from] target::TargetError),
    #[error("Unable to start window capture {0}")]
    UnableToStartCapture(#[from] windows_capture::capture::GraphicsCaptureApiError<WindowError>),
}

type Control = windows_capture::capture::CaptureControl<WindowCapture, WindowError>;

impl WindowCaptureHandler {
    fn new(target: target::Targets) -> Result<Self, WindowCaptureError> {
        let target = target.to_native()?;

        let capture_controller = match target {
            target::NativeTarget::Window(window) => {
                let settings = windows_capture::settings::Settings::new(
                    window,
                    windows_capture::settings::CursorCaptureSettings::Default,
                    windows_capture::settings::DrawBorderSettings::Default,
                    windows_capture::settings::ColorFormat::Bgra8,
                    (),
                );
                WindowCapture::start_free_threaded(settings)
            }
            target::NativeTarget::Monitor(monitor) => {
                let settings = windows_capture::settings::Settings::new(
                    monitor,
                    windows_capture::settings::CursorCaptureSettings::Default,
                    windows_capture::settings::DrawBorderSettings::WithoutBorder,
                    windows_capture::settings::ColorFormat::Bgra8,
                    (),
                );
                WindowCapture::start_free_threaded(settings)
            }
        }?;

        Ok(WindowCaptureHandler {
            control: capture_controller,
        })
    }
}

struct WindowCapture;

type WindowError = String;

impl GraphicsCaptureApiHandler for WindowCapture {
    type Flags = ();

    type Error = WindowError;

    fn new(ctx: windows_capture::capture::Context<Self::Flags>) -> Result<Self, Self::Error> {
        Ok(WindowCapture)
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut windows_capture::frame::Frame,
        _capture_control: windows_capture::graphics_capture_api::InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        println!("time: {:?}", frame.timespan());
        println!("width: {:?}", frame.width());
        println!("height: {:?}", frame.height());
        println!("color_format: {:?}", frame.color_format());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::thread::sleep;

    use super::*;

    #[test]
    fn test_window_capture() {
        let monitor = target::Targets::Window(target::Targets::get_foreground_window().unwrap());
        let window_capture = WindowCaptureHandler::new(monitor).unwrap();
        sleep(std::time::Duration::from_secs(5));
        window_capture.control.stop().unwrap();
    }
}
