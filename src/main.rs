use gtk4::{Application, ApplicationWindow, FlowBox, gdk::Paintable, prelude::*};
use std::{fs, path::Path, path::PathBuf};

fn main() {
    let app = Application::builder()
        .application_id("com.gluck.main")
        .build();

    app.connect_activate(build_ui);

    app.run();
}

struct _Song {
    path: PathBuf,
    title: String,
    album: String,
    album_artist: String,
}

pub struct Album {
    pub dir: PathBuf,
    pub title: String,
    pub artist: String,
    pub album_art: gtk4::gdk::Paintable, // apparently this doesnt work with gdk::Texture. this crate is shit
}

fn build_ui(app: &Application) {
    let rows_container = gtk4::Box::new(gtk4::Orientation::Vertical, 4);

    let page_menu_container = gtk4::Box::new(gtk4::Orientation::Vertical, 4);

    // Albums page
    //
    //

    let albums_grid = gtk4::FlowBox::new();
    albums_grid.set_valign(gtk4::Align::Start);
    // albums_grid.set_max_children_per_line(10);
    albums_grid.set_selection_mode(gtk4::SelectionMode::None);
    albums_grid.set_row_spacing(10);
    albums_grid.set_column_spacing(10);

    let scroller = gtk4::ScrolledWindow::new();
    scroller.set_child(Some(&albums_grid));

    scroller.set_vexpand(true);
    albums_grid.set_hexpand(true);
    albums_grid.set_vexpand(true);

    // dividers
    let v_div = gtk4::Separator::new(gtk4::Orientation::Vertical);
    let h_div = gtk4::Separator::new(gtk4::Orientation::Horizontal);

    // side menu buttons
    let page_button_albums = gtk4::Button::with_label("Albums");
    page_button_albums.set_size_request(80, 40);
    page_button_albums.connect_clicked(move |_| {
        switch_page("albums", &albums_grid);
    });

    let page_button_playlists = gtk4::Button::with_label("Playlists");
    page_button_playlists.set_size_request(80, 40);
    page_button_playlists.connect_clicked(move |_| {
        //switch_page("playlists");
    });

    let page_button_settings = gtk4::Button::with_label("Settings");
    page_button_settings.set_size_request(80, 40);
    page_button_settings.connect_clicked(move |_| {
        //switch_page("settings");
    });

    // top ribbon
    let ribbon = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);

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
    // WHY DOES THIS NOT EXPHAND VERTICALLY I HATE YOU GTK

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
    // why am i like this

    //
    //
    // adding to containers

    page_menu_container.append(&page_button_albums);
    page_menu_container.append(&page_button_playlists);
    page_menu_container.append(&page_button_settings);

    let main_column_container = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);

    main_column_container.append(&page_menu_container);
    main_column_container.append(&v_div);
    main_column_container.append(&scroller);

    rows_container.append(&ribbon);
    rows_container.append(&h_div);
    rows_container.append(&main_column_container);

    // flowboxes are for nerds

    fn switch_page(page: &str, albums_grid: &gtk4::FlowBox) {
        println!("{}", page);
        if page == "albums" {
            collect_album_lib(
                dirs::home_dir().unwrap().join("Music").as_path(),
                albums_grid,
            );
            // peak spaghetti code
        } else if page == "playlists" {
            // e
        } else if page == "settings" {
            //
        }
    }

    // create the main window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Gluck: the best gluck that ever glucked")
        .default_width(1000)
        .default_height(600)
        .child(&rows_container)
        .build();

    window.present(); // this comment eused to be useful, RIP that comment
}

fn collect_album_lib(music_dir: &Path, albums_grid: &FlowBox) {
    let mut lib: Vec<Album> = Vec::new();

    for entry in fs::read_dir(music_dir).unwrap_or_else(|_| panic!("Failed to read dir")) {
        if let Ok(entry) = entry {
            if let Ok(album) = load_album_info(&entry.path()) {
                lib.push(album);
            }
        }
    } // idk why tf nvim keeps getting mad at me for this, but im going to pretend there isnt a reason

    for album in lib {
        let album_container = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        let make_the_album_a_button_because_im_dumb = gtk4::Button::new();
        make_the_album_a_button_because_im_dumb.set_size_request(180, 180);
        let album_art_image = gtk4::Image::from_paintable(Some(&album.album_art));
        album_art_image.set_pixel_size(150); // make image but big
        let album_title_label = gtk4::Label::new(Some(&album.title));
        album_title_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        album_title_label.set_max_width_chars(15);
        let album_artist_label = gtk4::Label::new(Some(&album.artist));
        album_artist_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        album_artist_label.set_max_width_chars(15);

        album_container.append(&album_art_image);
        album_container.append(&album_title_label);
        album_container.append(&album_artist_label);
        make_the_album_a_button_because_im_dumb.set_child(Some(&album_container));
        make_the_album_a_button_because_im_dumb.set_hexpand(false);
        make_the_album_a_button_because_im_dumb.set_vexpand(false);

        albums_grid.append(&make_the_album_a_button_because_im_dumb);
    }
}

fn first_image_in_dir(dir: &Path) -> Option<PathBuf> {
    // another budget isaac budget solution
    let entries = fs::read_dir(dir).ok()?;
    for entry in entries {
        let entry = entry.ok()?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        match path.extension().and_then(|e| e.to_str()) {
            //ty to gpt for this long ass .sequence
            Some(ext) if matches!(ext.to_lowercase().as_str(), "png" | "jpg" | "jpeg") => {
                return Some(path);
            }
            _ => {}
        }
    }
    None
}

fn load_album_info(dir: &Path) -> Result<Album, String> {
    // loading the album art (breaks if there is none i cba with error handeling)
    let file =
        first_image_in_dir(dir).ok_or_else(|| format!("No album image found in {:?}", dir))?;
    let album_art_file = gtk4::gio::File::for_path(&file);

    let album_art = gtk4::gdk::Texture::from_file(&album_art_file)
        .map_err(|e| format!("couldnt load album art:: {}. your probably just stupid", e))?
        .upcast::<Paintable>();

    // picks a random file to use metadata for (im lazy as shit)
    let arbitrary_song_file = fs::read_dir(dir)
        .map_err(|e| format!("couldnt read directory: {}", e))?
        .filter_map(|e| e.ok())
        .find_map(|entry| {
            let path = entry.path();
            let ext = path.extension()?.to_str()?.to_lowercase();
            if matches!(ext.as_str(), "ogg" | "mp3") {
                Some(path)
            } else {
                None
            }
        })
        .ok_or_else(|| format!("No audio file found in {:?}", dir))?;

    // load song tags
    let song_file = taglib::File::new(arbitrary_song_file.to_str().unwrap())
        .map_err(|e| format!("Could not open audio file: {:?}", e))?;

    let tag = song_file
        .tag()
        .map_err(|e| format!("Could not get tags from audio file: {:?}", e))?;

    let song_title = tag
        .album()
        .unwrap_or("Unknown Title".to_string())
        .to_string();
    let song_artist = tag
        .artist()
        .unwrap_or("Unknown Artist".to_string())
        .to_string();

    // return all this bs into the album structure
    Ok(Album {
        dir: dir.to_path_buf(),
        title: song_title,
        artist: song_artist,
        album_art,
    })
}
