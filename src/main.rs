use core::f64;
use crossbeam_channel::{Receiver, Sender, bounded, unbounded};
use glib::{Bytes, timeout_add_local};
use gtk4::{Application, ApplicationWindow, FlowBox, gdk::Paintable, prelude::*};
use libmpv2::Mpv;
use rand::{rng, seq::SliceRandom};
use std::{
    fs,
    io::BufRead,
    path::{Path, PathBuf},
    rc::Rc,
    sync::{Arc, atomic::{AtomicBool, AtomicI32, Ordering}},
    thread,
    time::Duration,
};

static IS_SHUFFLED: AtomicBool = AtomicBool::new(false);
static MISSING_ALBUM_ICON: &[u8] = include_bytes!("./assets/missing_album_art.png");

fn set_shuffled(state: bool) {
    IS_SHUFFLED.store(state, Ordering::SeqCst);
}

fn get_shuffled() -> bool {
    IS_SHUFFLED.load(Ordering::SeqCst)
}

static TRACK_PROGRESS: AtomicI32 = AtomicI32::new(0);

fn set_track_progress(value: i32) {
    TRACK_PROGRESS.store(value, Ordering::SeqCst);
}

fn get_track_progress() -> i32 {
    let progress: i32 = TRACK_PROGRESS.load(Ordering::SeqCst);
    //println!("tarck progress retrieved as: {}", progress);
    progress
}

fn main() {
    let temp_playlist_ex = [
        "hello".to_string(),
        "/home/verbal/Music/Calamity OST/Fly Of Beelzebub.ogg".to_string(),
        "/home/verbal/Music/Deltarune/11. Cyber Battle (Solo) (DELTARUNE Chapter 2 Soundtrack) - Toby Fox.ogg".to_string(),
    ];
    let temp_content = temp_playlist_ex.join("\n");

    let _ = std::fs::create_dir_all(
        dirs::data_dir()
            .unwrap()
            .as_path()
            .join("Gluck")
            .join("Playlists"),
    );


    let _ = std::fs::File::create(
        dirs::data_dir()
            .unwrap()
            .as_path()
            .join("Gluck")
            .join("Playlists")
            .join(temp_playlist_ex[0].clone()),
    );
    let hh = std::fs::write(
        dirs::data_dir()
            .unwrap()
            .as_path()
            .join("Gluck")
            .join("Playlists")
            .join(temp_playlist_ex[0].clone()),
        temp_content,
    );
    if let Err(e) = hh {
        eprint!("{}", e);
    };
    // my old enemy, touples
    let (tx, rx) = unbounded::<PlayerCommand>();

    thread::spawn(move || {
        audio_thread(rx);
    });
    // way easier than python lol

    let player = Rc::new(Player { command_tx: tx });

    let app = Application::builder()
        .application_id("com.gluck.main")
        .build();

    let player_clone = player.clone();

    app.connect_activate(move |app| {
        build_ui(
            app,
            player_clone.clone(),
            dirs::data_dir().unwrap().as_path().join("Gluck"),
        )
    });

    app.run();
}

// Command the ui can send to the audio thread
pub enum PlayerCommand {
    Play(Vec<Song>, usize),
    Pause,
    Resume,
    Stop,
    TogggleShuffle,
    Forward,
    GetTrackDuration(Sender<f64>),
}

pub struct Player {
    command_tx: Sender<PlayerCommand>,
}

fn start_progress_updates(progress_bar: gtk4::ProgressBar, player: Rc<Player>) {
    timeout_add_local(Duration::from_millis(750), move || {
        let (reply_tx, reply_rx) = bounded(1);

        let _ = player
            .command_tx
            .send(PlayerCommand::GetTrackDuration(reply_tx));

        if let Ok(duration) = reply_rx.recv_timeout(Duration::from_millis(50)) {
            // println!("yay");
            let progress = get_track_progress();

            if duration > 0.0 {
                println!("pfraction: {} \nduration: {} \nprogress: {}\n", (progress  as f64 / duration), duration, progress);
                progress_bar.set_fraction(progress  as f64 / duration);
            }
        } else {
            println!("fucked");
        }

        glib::ControlFlow::Continue // keep running
    });
}

fn update_track_progress(mpv: &Arc<Mpv>) {
    while mpv.get_property::<f64>("duration").unwrap_or(0.0) != (mpv.get_time_ns() as f64) / 1_000_000_000.0 {
        // println!("progress set as {}", mpv.get_time_us().to_string());
        set_track_progress((mpv.get_time_ns() / 1_000_000_000) as i32);
    }
}

fn audio_thread(command_rx: Receiver<PlayerCommand>) {
    let mpv = Arc::new(Mpv::new().expect("Cant mpv, try harder next time"));

    mpv.set_property("pause", true).ok();

    while let Ok(command) = command_rx.recv() {
        match command {
            PlayerCommand::Play(queue, index) => {
                mpv.command("stop", &[]).ok();
                mpv.command("playlist-clear", &[]).ok();

                let mut queue_vec = vec![];
                println!("{:?}", queue_vec);

                if get_shuffled() {
                    queue_vec = queue.clone();
                    let first = queue_vec[index].clone();
                    let mut rng = rng();
                    queue_vec.shuffle(&mut rng);
                    queue_vec.insert(0, first);
                } else {
                    queue_vec = queue[index..queue.len()].to_vec();
                }

                for (i, song) in queue_vec.iter().enumerate() {
                    let path = song.path.to_string_lossy().to_string();
                    println!("Added file: {} to queue", song.title);
                    if i == 0 {
                        mpv.command("loadfile", &[&path]).ok();
                    } else {
                        mpv.command("loadfile", &[&path, "append"]).ok();
                    }
                }
                let mpv_clone = mpv.clone();
                    thread::spawn(move || {
                        update_track_progress(&mpv_clone);
                    });

                // unpause it because apparently its paused by default
                mpv.set_property("pause", false).ok();
            }

            PlayerCommand::Pause => {
                mpv.set_property("pause", true).ok();
            }

            PlayerCommand::Resume => {
                mpv.set_property("pause", false).ok();
            }

            PlayerCommand::Forward => {
                mpv.command("playlist-next", &[]).ok();
            }

            PlayerCommand::Stop => {
                mpv.command("stop", &[]).ok();
            }

            PlayerCommand::TogggleShuffle => {
                if get_shuffled() {
                    set_shuffled(false);
                    println!("unshuffled");
                } else {
                    set_shuffled(true);
                    println!("shuffled");
                }
            }

            PlayerCommand::GetTrackDuration(reply_tx) => {
                let duration = mpv.get_property::<f64>("duration").unwrap_or(0.0);

                let _ = reply_tx.send(duration);
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
    pub track_num: i16, // clear existing album buttons before reloading
}

pub struct Album {
    pub dir: PathBuf,
    pub title: String,
    pub artist: String,
    pub album_art: gtk4::gdk::Paintable,
}

fn build_ui(app: &Application, player: Rc<Player>, data_dir: PathBuf) {
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

    let playlist_list_container = Rc::new(gtk4::ListBox::new());
    let playlist_list_scroller = gtk4::ScrolledWindow::new();
    playlist_list_scroller.set_child(Some(&*playlist_list_container));
    playlist_list_scroller.set_vexpand(true);
    playlist_list_scroller.set_hexpand(true);

    // add albums view to ui stack
    main_content_stack.add_titled(&albums_scroller, Some("albums_view"), "Albums");

    let track_list_container = Rc::new(gtk4::ListBox::new());
    let track_scroller = gtk4::ScrolledWindow::new();
    track_scroller.set_child(Some(&*track_list_container));
    track_scroller.set_vexpand(true);
    track_scroller.set_hexpand(true);

    // add track list to ui stack
    main_content_stack.add_titled(&track_scroller, Some("track_view"), "Track List");
    main_content_stack.add_titled(&playlist_list_scroller, Some("playlists_view"), "Playlists");

    // side menu (page switching)
    let page_menu_container = gtk4::Box::new(gtk4::Orientation::Vertical, 4);

    // dividers
    let v_div = gtk4::Separator::new(gtk4::Orientation::Vertical);
    let h_div = gtk4::Separator::new(gtk4::Orientation::Horizontal);

    let main_column_container = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);

    // side menu buttons
    let stack_ref = main_content_stack.clone();
    let stack_ref_2 = main_content_stack.clone();
    let page_button_albums = gtk4::Button::with_label("Albums");
    page_button_albums.set_size_request(80, 40);
    page_button_albums.connect_clicked(move |_| {
        switch_page("albums", &stack_ref);
    });

    let main_content_stack_clone = main_content_stack.clone();
    let playlists_player = player.clone();
    let page_button_playlists = gtk4::Button::with_label("Playlists");
    page_button_playlists.set_size_request(80, 40);
    page_button_playlists.connect_clicked(move |_| {
        switch_page("playlists", &stack_ref_2);
        collect_playlists(
            main_content_stack_clone.clone(),
            playlist_list_container.clone(),
            data_dir.clone(),
            playlists_player.clone(),
        );
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

    let ribbon = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);

    let player_pause = player.clone();
    let ribbon_button_pause = gtk4::ToggleButton::with_label("Pause");
    ribbon_button_pause.set_size_request(40, 40);
    ribbon_button_pause.connect_toggled(move |btn| {
        if btn.is_active() {
            if let Err(e) = player_pause.command_tx.send(PlayerCommand::Pause) {
                eprintln!("Failed to send Pause command: {}", e);
            }
        } else {
            if let Err(e) = player_pause.command_tx.send(PlayerCommand::Resume) {
                eprintln!("Failed to send Resume command: {}", e);
            }
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

    let ribbon_button_forward = gtk4::Button::with_label("Forward");
    ribbon_button_forward.set_size_request(40, 40);
    ribbon_button_forward.connect_clicked({
        let player_forward = player.clone();
        move |_| {
            if let Err(e) = player_forward.command_tx.send(PlayerCommand::Forward) {
                eprintln!("I cant fucking do it: {}", e);
            }
        }
    });
    let ribbon_progress_bar = gtk4::ProgressBar::new();

    ribbon_progress_bar.set_show_text(true);
    ribbon_progress_bar.set_fraction(0.0);
    ribbon_progress_bar.set_text(Some(
        "This is the track name (trust me bro)".to_string().as_str(),
    ));
    ribbon_progress_bar.set_hexpand(true);
    start_progress_updates(ribbon_progress_bar.clone(), player.clone());

    let player_shuffle = player.clone();
    let ribbon_toggle_shuffle = gtk4::Button::with_label("Shuffle");
    ribbon_toggle_shuffle.set_size_request(40, 40);
    ribbon_toggle_shuffle.connect_clicked(move |_| {
        if let Err(e) = player_shuffle
            .command_tx
            .send(PlayerCommand::TogggleShuffle)
        {
            eprintln!("somit with shuffle: {}", e)
        }
    });

    let ribbon_label_progress = gtk4::Label::new(Some("00:00 / 00:00"));

    ribbon.append(&ribbon_button_pause);
    ribbon.append(&ribbon_button_stop);
    ribbon.append(&ribbon_button_forward);
    ribbon.append(&ribbon_progress_bar);
    ribbon.append(&ribbon_toggle_shuffle);
    ribbon.append(&ribbon_label_progress);

    page_menu_container.append(&page_button_albums);
    page_menu_container.append(&page_button_playlists);
    page_menu_container.append(&page_button_settings);

    main_column_container.append(&page_menu_container);
    main_column_container.append(&v_div);
    main_column_container.append(&*main_content_stack.clone()); // add the stack

    rows_container.append(&ribbon);
    rows_container.append(&h_div);
    rows_container.append(&main_column_container);

    fn switch_page(page: &str, stack: &gtk4::Stack) {
        println!("Switching to: {}", page);
        if page == "albums" {
            stack.set_visible_child_name("albums_view");
        } else if page == "tracks" {
            stack.set_visible_child_name("track_view");
        } else if page == "playlists" {
            stack.set_visible_child_name("playlists_view");
        }
    }

    // load teh albums page to start with because otherwise its blank
    collect_album_lib(
        dirs::audio_dir().unwrap().as_path(),
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
        if let Ok(album) = load_album_info(&entry.unwrap().path()) {
            lib.push(album);
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
                &player_ref.clone(), // Pass player reference down, again because im stupid and
                // forgot to make it a global xD
                true,
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
    player: &Rc<Player>, // Added Player
    move_window: bool,
) {
    while let Some(child) = track_list_container.last_child() {
        track_list_container.remove(&child);
    }

    let track_list_ref = track_list.clone();

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
        // disk number isnt real (literally)
        let label_track_num = gtk4::Label::new(Some(&track.track_num.to_string()));
        label_track_num.set_size_request(40, 20);

        track_container.append(&label_title);
        // track_container.append(&label_disk);
        track_container.append(&label_track_num);
        track_container.append(&label_duration);
        make_the_track_button_lol.set_child(Some(&track_container));

        let player_ref_button = player.clone(); // Clone for button closure
        let button_track_list_ref = track_list_ref.clone();

        make_the_track_button_lol.connect_clicked(move |_| {
            let track_list_ref_2 = button_track_list_ref.clone();
            let track_list_ref_3 = track_list_ref_2.clone();
            let index = track_list_ref_2.iter().position(|x| x.title == track.title);

            if let Err(e) = player_ref_button
                .command_tx
                .send(PlayerCommand::Play(track_list_ref_3, index.unwrap()))
            {
                eprintln!("Failed to send Play command: {}", e);
            };
        });

        row.set_child(Some(&make_the_track_button_lol));
        track_list_container.append(&row);

        //main_content_stack.set_visible_child_name("track_view");
    }
    if move_window {
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
    let image_path = first_image_in_dir(dir);
    //println!("{:?}", image_path);
    let album_art: Paintable;

    if image_path.is_none() {
        album_art = gtk4::gdk::Texture::from_bytes(&Bytes::from_static(MISSING_ALBUM_ICON))
            .map_err(|e| format!("AAAAAAAAAAAHHHHHHHHHhhh: {}. your probably just stupid", e))?
            .upcast::<Paintable>();
    } else {
        let file = image_path.ok_or_else(|| format!("No album image found in {:?}", dir))?;
        let album_art_file = gtk4::gio::File::for_path(&file);

        album_art = gtk4::gdk::Texture::from_file(&album_art_file)
            .map_err(|e| format!("couldnt load album art: {}. your probably just stupid", e))?
            .upcast::<Paintable>();
    }

    let arbitrary_song_file = fs::read_dir(dir)
        .map_err(|e| format!("couldnt read directory: {}", e))?
        .filter_map(|e| e.ok())
        .find_map(|entry| {
            let path = entry.path();
            let ext = path.extension()?.to_str()?.to_lowercase();
            if matches!(ext.as_str(), "ogg" | "mp3" | "flac") {
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
        .unwrap_or(
            dir.file_name()
                .unwrap()
                .to_os_string()
                .into_string()
                .unwrap(),
        )
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
        let path = entry.unwrap().path();
        let ext = path.extension().unwrap();
        if ext == "mp3" || ext == "ogg" {
            let song_file = taglib::File::new(path.to_str().unwrap()).unwrap();
            let tag = song_file.tag().unwrap();
            let duration = song_file.audioproperties().map(|p| p.length()).unwrap_or(0);
            let path_ref = path.clone();
            let song = Song {
                album: tag.album().unwrap_or("Unknown Album".to_string()),
                album_artist: tag.artist().unwrap_or("Unknown Artist".to_string()),
                path: path,
                title: tag.title().unwrap_or(
                    path_ref
                        .file_name()
                        .unwrap()
                        .to_os_string()
                        .into_string()
                        .unwrap(),
                ),
                duration: duration,
                disk: 0,
                track_num: (tag.track().map(|t| t as i16).unwrap_or(0)), // wtf is ts bro, stack overflow actually carrying my ass
            };
            album_contents.push(song);
        }
    }
    album_contents.sort_by_key(|song| song.track_num);

    album_contents
}

fn collect_playlists(
    main_content_stack: Rc<gtk4::Stack>,
    playlists_list_container: Rc<gtk4::ListBox>,
    data_dir: PathBuf,
    player: Rc<Player>,
) {
    // let example_playlist_format = [
    //     "hello",
    //     "/home/verbal/Music/Calamity OST/Fly Of Beelzebub.ogg",
    //     "/home/verbal/Music/One Shot/OneShot OST (Solstice) - Ghost in the Machine.ogg",
    //];
    for entry in fs::read_dir(data_dir.join("Playlists")).unwrap() {
        let playlists_list_container_clone = playlists_list_container.clone();
        let player_clone = player.clone();
        let main_content_stack_clone = main_content_stack.clone();
        let file = std::fs::OpenOptions::new()
            .read(true)
            .open(entry.unwrap().path());
        let reader = std::io::BufReader::new(file.unwrap());
        let mut playlist_data: Vec<String> = vec![];
        for line in reader.lines() {
            playlist_data.push(line.unwrap());
        }

        let playlist_label = gtk4::Label::new(Some(&playlist_data[0].as_str()));
        let mut track_list: Vec<Song> = vec![];
        for song_path in playlist_data {
            if song_path.chars().nth(0).unwrap() == "/".chars().next().unwrap() {
                println!("{}", song_path);
                let song_file = taglib::File::new(song_path.clone().as_str()).unwrap();
                let tag = song_file.tag().unwrap();
                let duration = song_file.audioproperties().map(|p| p.length()).unwrap_or(0);
                let song = Song {
                    album: tag.album().unwrap_or("Unknown Album".to_string()),
                    album_artist: tag.artist().unwrap_or("Unknown Artist".to_string()),
                    path: PathBuf::from(song_path.clone()),
                    title: tag.title().unwrap_or(
                        PathBuf::from(song_path)
                            .file_name()
                            .unwrap()
                            .to_os_string()
                            .into_string()
                            .unwrap(),
                    ),
                    duration: duration,
                    disk: 0,
                    track_num: (tag.track().map(|t| t as i16).unwrap_or(0)), // pasted this whole fucking section lol
                };
                track_list.push(song);
            }
        }
        let make_it_a_button_again_lmao_xd = gtk4::Button::new();
        make_it_a_button_again_lmao_xd.set_child(Some(&playlist_label));
        make_it_a_button_again_lmao_xd.connect_clicked(move |_| {
            instance_track_list(
                track_list.clone(),
                &playlists_list_container_clone,
                &main_content_stack_clone,
                &player_clone,
                false,
            );
        });
        make_it_a_button_again_lmao_xd.set_hexpand(true);
        let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 3);
        row.append(&make_it_a_button_again_lmao_xd);
        row.set_hexpand(true);
        playlists_list_container.append(&row);
        main_content_stack.set_visible_child_name("playlists_view");
    }
}
