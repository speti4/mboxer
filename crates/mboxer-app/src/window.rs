//! Where the main window stands at start-up (task 073).
//!
//! The configuration asks for a centred 1400x900 window. On a screen smaller
//! than that the system shrinks the window but leaves it where a 1400x900 one
//! would be centred, so part of it, the title bar buttons with it on macOS,
//! lies off the screen. [`fit_main_window`] puts such a window back: no larger
//! than the monitor's work area and centred in it. A window that already
//! stands inside the work area is left alone.

use tauri::{Manager, PhysicalPosition, PhysicalSize};

/// A rectangle in physical pixels: the position of its top left corner and its size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    fn right(&self) -> i64 {
        i64::from(self.x) + i64::from(self.width)
    }

    fn bottom(&self) -> i64 {
        i64::from(self.y) + i64::from(self.height)
    }

    fn holds(&self, other: &Rect) -> bool {
        other.x >= self.x
            && other.y >= self.y
            && other.right() <= self.right()
            && other.bottom() <= self.bottom()
    }
}

/// Where `window` (its outer rectangle) has to go to stand whole inside
/// `work`, the monitor's work area: `None` when it already does, otherwise the
/// window no larger than the work area and centred in it.
pub fn fit_into(window: Rect, work: Rect) -> Option<Rect> {
    if work.width == 0 || work.height == 0 || work.holds(&window) {
        return None;
    }
    let width = window.width.min(work.width);
    let height = window.height.min(work.height);
    // The differences are at most `u32::MAX / 2` after the halving, and a
    // monitor's size is far below that, so they fit an `i32`.
    let x = work.x.saturating_add(((work.width - width) / 2) as i32);
    let y = work.y.saturating_add(((work.height - height) / 2) as i32);
    Some(Rect {
        x,
        y,
        width,
        height,
    })
}

/// Moves the main window into the work area of its monitor when a part of it
/// lies outside. A window with no monitor to ask (it stands off every screen)
/// is measured against the primary one. Errors are ignored: the window then
/// stays where the system put it.
pub fn fit_main_window(app: &tauri::App) {
    let Some(window) = app.get_webview_window("main") else {
        eprintln!("fit: no main window");
        return;
    };
    eprintln!(
        "fit: current {:?} primary {:?} pos {:?} outer {:?} inner {:?} scale {:?}",
        window.current_monitor().map(|m| m.map(|m| *m.work_area())),
        window.primary_monitor().map(|m| m.map(|m| *m.work_area())),
        window.outer_position(),
        window.outer_size(),
        window.inner_size(),
        window.scale_factor(),
    );
    let monitor = match window.current_monitor() {
        Ok(Some(monitor)) => monitor,
        _ => match window.primary_monitor() {
            Ok(Some(monitor)) => monitor,
            _ => return,
        },
    };
    let (Ok(position), Ok(outer), Ok(inner)) = (
        window.outer_position(),
        window.outer_size(),
        window.inner_size(),
    ) else {
        return;
    };
    let area = monitor.work_area();
    let work = Rect {
        x: area.position.x,
        y: area.position.y,
        width: area.size.width,
        height: area.size.height,
    };
    let now = Rect {
        x: position.x,
        y: position.y,
        width: outer.width,
        height: outer.height,
    };
    let Some(fitted) = fit_into(now, work) else {
        eprintln!("fit: inside {now:?} {work:?}");
        return;
    };
    eprintln!("fit: {now:?} -> {fitted:?} in {work:?}");
    // `set_size` takes the size of the content: the title bar and the borders come off.
    let frame_width = outer.width.saturating_sub(inner.width);
    let frame_height = outer.height.saturating_sub(inner.height);
    if fitted.width != now.width || fitted.height != now.height {
        let _ = window.set_size(PhysicalSize::new(
            fitted.width.saturating_sub(frame_width),
            fitted.height.saturating_sub(frame_height),
        ));
    }
    let _ = window.set_position(PhysicalPosition::new(fitted.x, fitted.y));
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn rect(x: i32, y: i32, width: u32, height: u32) -> Rect {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn a_window_inside_the_work_area_stays() {
        let work = rect(0, 25, 1920, 1055);
        assert_eq!(fit_into(rect(260, 90, 1400, 900), work), None);
        // Touching the edges is inside.
        assert_eq!(fit_into(work, work), None);
    }

    #[test]
    fn a_shrunk_window_left_of_the_screen_moves_in() {
        // A 1024x768 screen: the system shrank the window to the work area's
        // size and left it where a 1400 wide one would be centred.
        let work = rect(0, 25, 1024, 674);
        assert_eq!(
            fit_into(rect(-188, 25, 1024, 674), work),
            Some(rect(0, 25, 1024, 674))
        );
    }

    #[test]
    fn a_window_larger_than_the_work_area_shrinks_and_is_centred() {
        let work = rect(0, 30, 1280, 720);
        assert_eq!(
            fit_into(rect(-60, -60, 1400, 900), work),
            Some(rect(0, 30, 1280, 720))
        );
        // Wider than the screen only: the height is kept and centred.
        assert_eq!(
            fit_into(rect(-60, 40, 1400, 600), work),
            Some(rect(0, 90, 1280, 600))
        );
    }

    #[test]
    fn a_work_area_of_a_second_monitor_keeps_its_offset() {
        let work = rect(-1280, 0, 1280, 1000);
        assert_eq!(
            fit_into(rect(-1400, 50, 1000, 800), work),
            Some(rect(-1140, 100, 1000, 800))
        );
    }

    #[test]
    fn an_empty_work_area_moves_nothing() {
        assert_eq!(fit_into(rect(-5, -5, 100, 100), rect(0, 0, 0, 0)), None);
    }
}
