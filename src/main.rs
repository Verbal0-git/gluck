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

    // dividers
    let v_div = gtk4::Separator::new(gtk4::Orientation::Vertical);
    let h_div = gtk4::Separator::new(gtk4::Orientation::Horizontal);

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

    let page_button_settings = gtk4::Button::with_label("Settings");
    page_button_settings.set_size_request(80, 40);
    page_button_settings.connect_clicked(move |_| {
        switch_page("settings");
    });

    //
    // top ribbon
    let ribbon = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    // let label = gtk4::Label::new(Some("Hello"));
    // label.set_size_request(80, 40);

    let ribbon_button_pause = gtk4::Button::with_label("Pause");
    ribbon_button_pause.set_size_request(40, 40);
    ribbon_button_pause.connect_clicked(move |_| {
        println!("Pause button pressed");
    });

    let ribbon_button_stop = gtk4::Button::with_label("Stop");
    ribbon_button_stop.set_size_request(40, 40);
    ribbon_button_stop.connect_clicked(move |_| {
        println!("Stop button pressed");
    });

    let ribbon_button_back = gtk4::Button::with_label("Back");
    ribbon_button_back.set_size_request(40, 40);
    ribbon_button_back.connect_clicked(move |_| {
        println!("Back button pressed");
    });

    let ribbon_button_forward = gtk4::Button::with_label("Forward");
    ribbon_button_forward.set_size_request(40, 40);
    ribbon_button_forward.connect_clicked(move |_| {
        println!("Forward button pressed");
    });

    let ribbon_progress_bar = gtk4::ProgressBar::new();
    ribbon_progress_bar.set_fraction(0.5);
    ribbon_progress_bar.set_text(Some("Fuck this stupid pice of shit"));
    ribbon_progress_bar.set_show_text(true);
    ribbon_progress_bar.set_size_request(600, 80);
    ribbon_progress_bar.set_vexpand(true);
    ribbon_progress_bar.set_valign(gtk4::Align::Fill);

    ribbon.append(&ribbon_button_pause);
    ribbon.append(&ribbon_button_stop);
    ribbon.append(&ribbon_button_back);
    ribbon.append(&ribbon_button_forward);
    ribbon.append(&ribbon_progress_bar);
    ribbon.set_vexpand(false);

    //
    //
    // adding to containers

    page_menu_container.append(&page_button_albums);
    page_menu_container.append(&page_button_playlists);
    page_menu_container.append(&page_button_settings);

    let main_column_container = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);

    main_column_container.append(&page_menu_container);
    main_column_container.append(&v_div);

    rows_container.append(&ribbon);
    rows_container.append(&h_div);
    rows_container.append(&main_column_container);

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
