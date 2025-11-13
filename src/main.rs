use gtk4::{Application, ApplicationWindow, prelude::*};
use regex::Regex;
use std::rc::Rc;

fn main() {
    let app = Application::builder()
        .application_id("com.gluck.main")
        .build();
    app.connect_activate(build_ui);

    app.run();
}

fn build_ui(app: &Application) {
    let rows_container = gtk4::Box::new(gtk4::Orientation::Vertical, 4);

    let page_menu_container = gtk4::Box::new(gtk4::Orientation::Vertical, 4);

    // side menu buttons
    let page_button_albums = gtk4::Button::with_label("Albums");
    page_button_albums.set_size_request(80, 40);
    page_button_albums.connect_clicked(move |_| {
        switch_page("albums");
    });

    let page_button_playlists = gtk4::Button::with_label("Playlists");
    page_button_playlists.set_size_request(80, 40);
    page_button_playlists.connect_clicked(move |_| {
        switch_page("playlists");
    });

    page_menu_container.append(&page_button_albums);
    page_menu_container.append(&page_button_playlists);
    rows_container.append(&page_menu_container);

    fn switch_page(page: &str) {
        println!("{}", page);
    }

    // Create the main window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Gluck Music Player")
        // .default_width(400)
        // .default_height(800)
        .child(&rows_container)
        .build();

    window.present(); // Show the window
}
