use gtk4::{Application, ApplicationWindow, gdk::Paintable, prelude::*};
use std::{fs, path::Path, path::PathBuf};
use taglib;

fn main() {
    let app = Application::builder()
        .application_id("com.gluck.main")
        .build();

    app.connect_activate(build_ui);

    app.run();
}

struct Song {
    path: String,
    title: String,
    album: String,
    album_artist: String,
}

pub struct Album {
    pub dir: PathBuf,
    pub title: String,
    pub artist: String,
    pub album_art: gtk4::gdk::Paintable,
}

fn build_ui(app: &Application) {
    let dir_music = Some("~/Music/");

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
    ribbon_progress_bar.set_size_request(400, 40);
    ribbon_progress_bar.set_hexpand(true);
    ribbon_progress_bar.set_vexpand(true);
    ribbon_progress_bar.set_valign(gtk4::Align::Center);

    let ribbon_toggle_shuffle = gtk4::Button::with_label("Shuffle");
    ribbon_toggle_shuffle.set_size_request(40, 40);
    ribbon_toggle_shuffle.connect_clicked(move |_| {
        println!("Shuffle");
    });

    let ribbon_label_progress = gtk4::Label::builder()
        .height_request(40)
        .margin_end(5)
        .label("00:00/00:00")
        .build();

    ribbon.append(&ribbon_button_pause);
    ribbon.append(&ribbon_button_stop);
    ribbon.append(&ribbon_button_back);
    ribbon.append(&ribbon_button_forward);
    ribbon.append(&ribbon_progress_bar);
    ribbon.append(&ribbon_toggle_shuffle);
    ribbon.append(&ribbon_label_progress);
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

fn collect_album_lib(music_dir: String) {
    let lib: Vec<Album> = Vec::new();
    for entry in fs::read_dir(music_dir).unwrap() {
        let album_path = entry.unwrap();
        // lib.append(load_album_info(entry))
    }

    fn load_album_info(dir: &Path) -> Result<Album, String> {
        let file = gio::File::for_path(dir);
        let arbitrary_song_path = Path::new("~/Music/Outer Wilds/01 Timber Hearth.ogg");
        let arbitrary_song = taglib::File::new(arbitrary_song_path)
            .map_err(|e| format!("Failed to open or parse file: {}", e))?;

        let tags = arbitrary_song
            .tag()
            .map_err(|e| format!("File does not contain readable tags: {}", e))?;

        let artist = tags.artist();
        let album = tags.album();

        let album_art = gtk4::gdk::Texture::from_file(&file)
            .map_err(|e| format!("Failed to load image into Texture: {}", e))?
            .upcast::<Paintable>();

        // Return struct instance
        Ok(Album {
            dir: dir.to_path_buf(),
            title: "e".to_owned(),
            artist: "h".to_owned(),
            album_art: album_art,
        })
    }
}
