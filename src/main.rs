// Extended from winit example from wry
// https://github.com/tauri-apps/wry/blob/dev/examples/winit.rs

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};
use wry::WebViewBuilder;

fn main() -> wry::Result<()> {
    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
    ))]
    {
        use gtk::prelude::DisplayExtManual;

        gtk::init().unwrap();
        if gtk::gdk::Display::default().unwrap().backend().is_wayland() {
            panic!("This example doesn't support wayland!");
        }

        // we need to ignore this error here otherwise it will be catched by winit and will be
        // make the example crash
        winit::platform::x11::register_xlib_error_hook(Box::new(|_display, error| {
            let error = error as *mut x11_dl::xlib::XErrorEvent;
            (unsafe { (*error).error_code }) == 170
        }));
    }

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::LogicalSize::new(800, 800))
        .build(&event_loop)
        .unwrap();

    let request_webview_hide = Arc::new(AtomicBool::new(false));
    let rwh = request_webview_hide.clone();

    #[allow(unused_mut)]
    let mut builder = WebViewBuilder::new(&window);
    let webview = builder
        .with_html(
            r#"
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Document</title>
  </head>
  <body>
    <h1>This is a webview.</h1>
    <p>
      Press space to toggle visibility.
    </p>
    <script>
      window.addEventListener('keydown', (e) => {
        if (e.code === 'Space') {
          window.ipc.postMessage('keydown');
        }
      });
    </script>
  </body>
</html>
"#,
        )
        .with_visible(false)
        .with_focused(false)
        .with_ipc_handler(move |_| {
            println!("Webview Space Event");
            rwh.store(true, Ordering::Relaxed);
        })
        .build()?;

    event_loop
        .run(move |event, evl| {
            evl.set_control_flow(ControlFlow::Poll);

            #[cfg(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd",
            ))]
            while gtk::events_pending() {
                gtk::main_iteration_do(false);
            }

            if request_webview_hide.load(Ordering::Relaxed) {
                webview.set_visible(false).unwrap();
                request_webview_hide.store(false, Ordering::Relaxed);
            }

            match event {
                #[cfg(any(
                    target_os = "linux",
                    target_os = "dragonfly",
                    target_os = "freebsd",
                    target_os = "netbsd",
                    target_os = "openbsd",
                ))]
                Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    use wry::dpi::{PhysicalPosition, PhysicalSize};

                    _webview
                        .set_bounds(wry::Rect {
                            position: PhysicalPosition::new(0, 0).into(),
                            size: PhysicalSize::new(size.width, size.height).into(),
                        })
                        .unwrap();
                }
                Event::WindowEvent {
                    event: WindowEvent::KeyboardInput { event, .. },
                    ..
                } => {
                    if event.state.is_pressed()
                        && event.physical_key == PhysicalKey::Code(KeyCode::Space)
                    {
                        println!("Winit Space Event");
                        webview.set_visible(true).unwrap();
                        webview.focus().unwrap();
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => evl.exit(),
                _ => {}
            }
        })
        .unwrap();

    Ok(())
}
