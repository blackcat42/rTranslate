use fltk::{
    app,
    dialog
};



pub fn app_message(e: &str) {
    let pos = screen_center();
    dialog::alert(pos.0 - 210, pos.1 - 40, e);
    
}

pub fn screen_center() -> (i32, i32) {
    (
        (app::screen_size().0 / 2.0) as i32,
        (app::screen_size().1 / 2.0) as i32,
    )
}