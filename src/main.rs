use crossbeam_channel::{Receiver, Sender, unbounded};
use gtk4::{Application, ApplicationWindow, FlowBox, gdk::Paintable, prelude::*};
use rodio::{Decoder, OutputStreamBuilder, Sink}; // Removed unused OutputStream
use std::{
    fs,
    path::{Path, PathBuf},
    rc::Rc,
    thread,
};

fn main() {
    // my old enemy, touples
    let (tx, rx) = unbounded::<PlayerCommand>();

    // way easier than python lol
    thread::spawn(move || {
        audio_thread(rx);
    });

    let player = Rc::new(Player { command_tx: tx });

    let app = Application::builder()
        .application_id("com.gluck.main")
        .build();

    let player_clone = player.clone();
    app.connect_activate(move |app| build_ui(app, player_clone.clone()));

    app.run();
}

// Command the ui can send to the audio thread
pub enum PlayerCommand {
    Play(PathBuf),
    Pause,
    Resume,
    Stop,
}

pub struct Player {
    command_tx: Sender<PlayerCommand>,
}

fn audio_thread(command_rx: Receiver<PlayerCommand>) {
    // let rodio exist
    let stream = match rodio::OutputStreamBuilder::open_default_stream() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Commited sudoku: {}", e);
            return;
        }
    };

    let sink = rodio::Sink::connect_new(&stream.mixer());
    sink.play();

    loop {
        match command_rx.recv() {
            Ok(command) => match command {
                PlayerCommand::Play(path) => {
                    println!("Playing {:?}", path);
                    sink.clear();

                    // load and play a file
                    match std::fs::File::open(&path) {
                        Ok(file) => {
                            let buffered = std::io::BufReader::new(file);
                            match rodio::Decoder::new(buffered) {
                                Ok(source) => {
                                    sink.append(source);

                                    if sink.empty() {
                                        eprintln!("Decoder probably shat themselves");
                                    } else {
                                        println!(
                                            "ive got a queue of: {}, your really giving me a hard time",
                                            sink.len()
                                        );
                                    }
                                    sink.play();
                                }
                                Err(e) => eprintln!("asjfjnl: {}", e),
                            }
                        }
                        Err(e) => eprintln!("Failed to open audio file {:?}: {}", path, e),
                    }
                }
                PlayerCommand::Pause => {
                    println!("Paused");
                    sink.pause();
                }
                PlayerCommand::Resume => {
                    println!("Resumed");
                    sink.play();
                }
                PlayerCommand::Stop => {
                    println!("Stopd");
                    sink.stop();
                    sink.clear();
                }
            },
            Err(e) => {
                println!("Oh dear, your fucked.. {}", e);
                break;
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Song {
    pub path: PathBuf,
    pub title: String,
    pub album: String,
    pub album_artist: String,
    pub duration: u32, // dont ask questions
    pub disk: i8,
    pub track_num: i16, // Clear existing album buttons before reloading
}

pub struct Album {
    pub dir: PathBuf,
    pub title: String,
    pub artist: String,
    pub album_art: gtk4::gdk::Paintable,
}

fn build_ui(app: &Application, player: Rc<Player>) {
    let rows_container = gtk4::Box::new(gtk4::Orientation::Vertical, 4);

    // the actuall stuff on screen, probably needed
    let main_content_stack = Rc::new(gtk4::Stack::new());
    main_content_stack.set_vexpand(true);
    main_content_stack.set_hexpand(true);

    let albums_grid = Rc::new(gtk4::FlowBox::new());
    albums_grid.set_valign(gtk4::Align::Start);
    albums_grid.set_selection_mode(gtk4::SelectionMode::None);
    albums_grid.set_row_spacing(2);
    albums_grid.set_column_spacing(2);

    let albums_scroller = gtk4::ScrolledWindow::new();
    albums_scroller.set_child(Some(&*albums_grid));
    albums_scroller.set_vexpand(true);
    albums_scroller.set_hexpand(true);

    // add albums view to ui stack
    main_content_stack.add_titled(&albums_scroller, Some("albums_view"), "Albums");

    let track_list_container = Rc::new(gtk4::ListBox::new());
    let track_scroller = gtk4::ScrolledWindow::new();
    track_scroller.set_child(Some(&*track_list_container));
    track_scroller.set_vexpand(true);
    track_scroller.set_hexpand(true);

    // add track list to ui stack
    main_content_stack.add_titled(&track_scroller, Some("track_view"), "Track List");

    // side menu (page switching)
    let page_menu_container = gtk4::Box::new(gtk4::Orientation::Vertical, 4);

    // dividers
    let v_div = gtk4::Separator::new(gtk4::Orientation::Vertical);
    let h_div = gtk4::Separator::new(gtk4::Orientation::Horizontal);

    let main_column_container = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);

    // side menu buttons
    let stack_ref = main_content_stack.clone();
    let page_button_albums = gtk4::Button::with_label("Albums");
    page_button_albums.set_size_request(80, 40);
    page_button_albums.connect_clicked(move |_| {
        switch_page("albums", &stack_ref);
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

    let page_button_track_list = gtk4::Button::with_label("Tracks");
    page_button_track_list.set_size_request(80, 40);
    page_button_track_list.connect_clicked(move |_| {
        //switch_page("tracks");
    });

    // top ribbon
    let ribbon = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);

    // Connect Play/Pause button
    let player_pause = player.clone();
    let ribbon_button_pause = gtk4::Button::with_label("Pause");
    ribbon_button_pause.set_size_request(40, 40);
    ribbon_button_pause.connect_clicked(move |_| {
        // NOTE: Make this a toggle later (if you forget i will shoot you)
        if let Err(e) = player_pause.command_tx.send(PlayerCommand::Pause) {
            eprintln!("i cannot be stopped: {}", e);
        }
    });
    let player_stop = player.clone();
    let ribbon_button_stop = gtk4::Button::with_label("Stop");
    ribbon_button_stop.set_size_request(40, 40);
    ribbon_button_stop.connect_clicked(move |_| {
        if let Err(e) = player_stop.command_tx.send(PlayerCommand::Stop) {
            eprintln!("jhbashbdhf: {}", e);
        }
    });

    let ribbon_button_back = gtk4::Button::with_label("Back");
    ribbon_button_back.set_size_request(40, 40);
    let ribbon_button_forward = gtk4::Button::with_label("Forward");
    ribbon_button_forward.set_size_request(40, 40);
    let ribbon_progress_bar = gtk4::ProgressBar::new();
    ribbon_progress_bar.set_fraction(0.5);
    ribbon_progress_bar.set_show_text(true);
    ribbon_progress_bar.set_size_request(400, 40);
    ribbon_progress_bar.set_hexpand(true);
    ribbon_progress_bar.set_valign(gtk4::Align::Center);
    ribbon_progress_bar.set_text(Some("fuck this stupid pice of shit"));
    // FUCKING EXPAND VERTICALLY BITCH

    let ribbon_toggle_shuffle = gtk4::Button::with_label("Shuffle");
    ribbon_toggle_shuffle.set_size_request(40, 40);
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

    // adding to containers
    page_menu_container.append(&page_button_albums);
    page_menu_container.append(&page_button_playlists);
    page_menu_container.append(&page_button_settings);

    main_column_container.append(&page_menu_container);
    main_column_container.append(&v_div);
    main_column_container.append(&*main_content_stack); // add the stack

    rows_container.append(&ribbon);
    rows_container.append(&h_div);
    rows_container.append(&main_column_container);

    fn switch_page(page: &str, stack: &gtk4::Stack) {
        println!("Switching to: {}", page);
        if page == "albums" {
            stack.set_visible_child_name("albums_view");
        } else if page == "tracks" {
            stack.set_visible_child_name("track_view");
        }
    }

    // load teh albums page to start with because otherwise its blank
    collect_album_lib(
        dirs::home_dir().unwrap().join("Music").as_path(),
        &albums_grid,
        track_list_container.clone(),
        main_content_stack.clone(), // pass the stack reference
        player.clone(),             // Pass player reference
    );
    switch_page("albums", &main_content_stack);

    // this eused to be a helpful comment. RIP that comment
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Gluck: the best gluck that ever glucked")
        .default_width(1000)
        .default_height(600)
        .child(&rows_container)
        .build();

    window.present();
}

fn collect_album_lib(
    music_dir: &Path,
    albums_grid: &FlowBox,
    track_list_container: Rc<gtk4::ListBox>,
    main_content_stack: Rc<gtk4::Stack>,
    player: Rc<Player>, // Added Player
) {
    let mut lib: Vec<Album> = Vec::new();

    while let Some(child) = albums_grid.last_child() {
        albums_grid.remove(&child);
    }

    for entry in fs::read_dir(music_dir).unwrap_or_else(|_| panic!("Failed to read dir")) {
        if let Ok(entry) = entry {
            if let Ok(album) = load_album_info(&entry.path()) {
                lib.push(album);
            }
        }
    }

    for album in lib {
        let album_container = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        let make_the_album_a_button_because_im_dumb = gtk4::Button::new();
        make_the_album_a_button_because_im_dumb.set_size_request(180, 180);

        let track_list_container_ref = track_list_container.clone();
        let main_content_stack_ref = main_content_stack.clone();
        let player_ref = player.clone(); // Clone player for the closure

        let album_dir = album.dir.clone(); // Clone the path for the closure

        make_the_album_a_button_because_im_dumb.connect_clicked(move |_| {
            // actually fetch teh track list
            let track_list = get_the_damn_track_list(&album_dir);
            instance_track_list(
                track_list,
                &track_list_container_ref,
                &main_content_stack_ref,
                player_ref.clone(), // Pass player reference down
            );
        });

        let album_art_image = gtk4::Image::from_paintable(Some(&album.album_art));
        album_art_image.set_pixel_size(150);
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

fn instance_track_list(
    track_list: Vec<Song>,
    track_list_container: &gtk4::ListBox,
    main_content_stack: &gtk4::Stack,
    player: Rc<Player>, // Added Player
) {
    while let Some(child) = track_list_container.last_child() {
        track_list_container.remove(&child);
    }

    for track in track_list {
        let row = gtk4::ListBoxRow::new();
        let make_the_track_button_lol = gtk4::Button::new();
        let track_container = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);

        let label_duration = gtk4::Label::new(Some(
            format!("{}:{}", track.duration / 60, track.duration % 60).as_str(),
        ));
        label_duration.set_size_request(40, 20);
        label_duration.set_margin_end(15);
        let label_title = gtk4::Label::new(Some(&track.title));
        label_title.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        label_title.set_max_width_chars(8);
        label_title.set_xalign(-1.0);
        label_title.set_hexpand(true);
        //let label_disk = gtk4::Label::new(Some(&track.disk.to_string()));
        //label_disk.set_size_request(40, 20);
        // disk number isnt real
        let label_track_num = gtk4::Label::new(Some(&track.track_num.to_string()));
        label_track_num.set_size_request(40, 20);

        track_container.append(&label_title);
        // track_container.append(&label_disk);
        track_container.append(&label_track_num);
        track_container.append(&label_duration);
        make_the_track_button_lol.set_child(Some(&track_container));

        let track_title = track.title.clone();
        let track_title_ref = track_title.clone();

        let track_ref = track.clone();
        let player_ref_button = player.clone(); // Clone for button closure
        let player_ref_row = player.clone(); // Clone for row closure

        make_the_track_button_lol.connect_clicked(move |_| {
            println!("now raping your ears with: {:?}", track_title);
            // Send the Play command to the audio thread
            if let Err(e) = player_ref_button
                .command_tx
                .send(PlayerCommand::Play(track_ref.path.clone()))
            {
                eprintln!("Failed to send Play command: {}", e);
            }
        });

        let track_ref_row = track.clone(); // Clone for row closure
        row.connect_activate(move |_| {
            println!("now raping your ears with: {:?}", track_title_ref);
            // Send the Play command to the audio thread
            if let Err(e) = player_ref_row
                .command_tx
                .send(PlayerCommand::Play(track_ref_row.path.clone()))
            {
                eprintln!("Failed to send Play command: {}", e);
            }
        });

        row.set_child(Some(&make_the_track_button_lol));
        track_list_container.append(&row);

        main_content_stack.set_visible_child_name("track_view");
    }
}

fn first_image_in_dir(dir: &Path) -> Option<PathBuf> {
    let entries = fs::read_dir(dir).ok()?;
    for entry in entries {
        let entry = entry.ok()?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        match path.extension().and_then(|e| e.to_str()) {
            Some(ext) if matches!(ext.to_lowercase().as_str(), "png" | "jpg" | "jpeg") => {
                return Some(path);
            }
            _ => {}
        }
    }
    None
}

fn load_album_info(dir: &Path) -> Result<Album, String> {
    let file =
        first_image_in_dir(dir).ok_or_else(|| format!("No album image found in {:?}", dir))?;
    let album_art_file = gtk4::gio::File::for_path(&file);

    let album_art = gtk4::gdk::Texture::from_file(&album_art_file)
        .map_err(|e| format!("couldnt load album art:: {}. your probably just stupid", e))?
        .upcast::<Paintable>();

    let arbitrary_song_file = fs::read_dir(dir)
        .map_err(|e| format!("couldnt read directory: {}", e))?
        .filter_map(|e| e.ok())
        .find_map(|entry| {
            let path = entry.path();
            let ext = path.extension()?.to_str()?.to_lowercase();
            if matches!(ext.as_str(), "ogg" | "mp3" | "flac" | "wav") {
                // if this doesnt support flac henery will murder me
                Some(path)
            } else {
                None
            }
        })
        .ok_or_else(|| format!("No audio file found in {:?}", dir))?;

    let song_file = taglib::File::new(arbitrary_song_file.to_str().unwrap())
        .map_err(|e| format!("Could not open the file: {:?} because ur stupid", e))?;

    let tag = song_file
        .tag()
        .map_err(|e| format!("oops, your tags are fucked >:3: {:?}", e))?;

    let song_title = tag
        .album()
        .unwrap_or("Unknown Title".to_string())
        .to_string();
    let song_artist = tag
        .artist()
        .unwrap_or("Unknown Artist".to_string())
        .to_string();

    Ok(Album {
        dir: dir.to_path_buf(),
        title: song_title,
        artist: song_artist,
        album_art,
    })
}

fn get_the_damn_track_list(album_path: &Path) -> Vec<Song> {
    let mut album_contents = Vec::new();
    for entry in fs::read_dir(album_path).unwrap() {
        if let Ok(entry) = entry {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext == "mp3" || ext == "ogg" {
                    let song_file = taglib::File::new(entry.path().to_str().unwrap()).unwrap();
                    let tag = song_file.tag().unwrap();
                    let duration = song_file.audioproperties().map(|p| p.length()).unwrap_or(0);
                    let song = Song {
                        album: tag.album().unwrap_or("Unknown Album".to_string()),
                        album_artist: tag.artist().unwrap_or("Unknown Artist".to_string()),
                        path: entry.path(),
                        title: tag.title().unwrap_or("Unknown Title".to_string()),
                        duration: duration,
                        disk: 0,
                        track_num: (tag.track().map(|t| t as i16).unwrap_or(0)), // wtf is ts bro, stack overflow actually carrying my ass
                    };
                    album_contents.push(song);
                }
            }
        }
    }
    album_contents.sort_by_key(|song| song.track_num);

    album_contents
}
