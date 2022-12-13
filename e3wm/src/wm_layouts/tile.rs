use crate::{
    layouts::Layouts,
    wm_layouts::max::max,
    x::{Connection, Window, WindowChanges},
};

pub fn tile(layouts: &mut Layouts, conn: &Connection) {
    if layouts.windows.is_empty() {
        return;
    }
    if layouts.windows.len() == 1 {
        max(layouts, conn);
        return;
    }
    let gaps: i16 = 5;
    let border_width: u16 = 5;
    let bar_height: i16 = 26;

    let master_width = (conn.screen_width as f32 * 0.5) as u16;
    let master_height = conn.screen_height as u16;

    let num_stack_win = layouts.windows.len() - 1;

    let mut track_y = 0;
    for win in 0..layouts.windows.len() {
        if win == 0 {
            let cwin = WindowChanges {
                x: 0,
                y: 0,
                width: master_width,
                height: master_height,
                border_width: 0,
            };
            conn.configure_window(&layouts.windows[win], &cwin, border_width, bar_height, gaps);
        } else if win == 1 {
            let cwin = WindowChanges {
                x: master_width as i16,
                y: track_y,
                width: master_width,
                height: master_height / num_stack_win as u16,
                border_width: 0,
            };
            conn.configure_window(&layouts.windows[win], &cwin, border_width, bar_height, gaps);
            track_y += master_height as i16 / num_stack_win as i16;
        } else {
            // println!("{}", track_y);
            let cwin = WindowChanges {
                x: master_width as i16,
                y: track_y - bar_height,
                width: master_width,
                height: master_height / num_stack_win as u16 + bar_height as u16,
                border_width: 0,
            };
            conn.configure_window(&layouts.windows[win], &cwin, border_width, bar_height, gaps);
            track_y += master_height as i16 / num_stack_win as i16;
        }
        layouts.windows[win] = Window::new(conn, layouts.windows[win].get_window());
        if layouts.focus_win == win {
            conn.configure_border(layouts.windows[layouts.focus_win].get_window(), true);
        } else {
            conn.configure_border(layouts.windows[win as usize].get_window(), false);
        }
    }
}
