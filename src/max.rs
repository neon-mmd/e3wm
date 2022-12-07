use crate::{
    layouts::Layouts,
    x::{Connection, WindowChanges},
};

pub fn max(layouts: &Layouts, conn: &Connection) {
    if layouts.windows.is_empty() {
        return;
    }

    let window_changes = WindowChanges {
        x: 0,
        y: 0,
        width: conn.screen_width,
        height: conn.screen_height,
        border_width: 0,
        // sibling: win,
        // stack_mode: xcb::x::StackMode::Above,
    };

    let gaps: i16 = 5;
    let border_width: u16 = 5;
    let bar_height: i16 = 26;
    for window in layouts.windows.iter() {
        // connection.stop_window_events(&window);
        // conn.map_window(&window);
        conn.configure_window(&window, &window_changes, border_width, bar_height, gaps);
        // connection.track_window_events(&window);
    }
    conn.configure_border(layouts.windows[layouts.focus_win].get_window(), true);
}
