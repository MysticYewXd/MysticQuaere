use gtk::prelude::*;
use gtk::{gio, glib, Application, ApplicationWindow, Image, Label, ListBox, Orientation, SearchEntry};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

const APP_ID: &str = "dev.mystic.Quaere";
const MAX_APPS: usize = 6;
const MAX_FILES: usize = 4;
const MAX_INDEXED_FILES: usize = 150_000;

// The look: a translucent rounded card that takes its colours from your
// wallpaper. Hyprland blurs whatever is behind it. The @mq_ colours are filled
// in by palette_css() below, and CARD_ALPHA by how dark the wallpaper is.
const CSS: &str = r#"
window.mystic {
    background: transparent;
}
window.mystic .card {
    background: linear-gradient(150deg,
        alpha(mix(@mq_surface, @mq_primary, 0.16), CARD_ALPHA),
        alpha(mix(@mq_surface, @mq_tertiary, 0.18), CARD_ALPHA));
    border: 1px solid alpha(@mq_primary, 0.24);
    border-radius: 20px;
    padding: 8px;
    box-shadow: inset 0 1px 0 alpha(@mq_text, 0.07);
}
window.mystic .card entry.search {
    background: transparent;
    border: none;
    box-shadow: none;
    outline: none;
    min-height: 52px;
    padding: 0 10px;
    font-size: 22px;
    color: @mq_text;
    caret-color: @mq_primary;
}
window.mystic .card entry.search image {
    color: alpha(@mq_primary, 0.80);
    -gtk-icon-size: 22px;
}
window.mystic .card entry.search text placeholder {
    color: alpha(@mq_text, 0.40);
}
window.mystic .card entry.search text selection {
    background: alpha(@mq_primary, 0.30);
    color: @mq_text;
}
window.mystic .card separator {
    background: linear-gradient(to right, alpha(@mq_primary, 0.40), alpha(@mq_tertiary, 0.40));
    min-height: 1px;
    margin: 4px 6px;
}
window.mystic .card list {
    background: transparent;
}
window.mystic .card row {
    border-radius: 12px;
    padding: 4px 6px;
    color: @mq_text;
}
window.mystic .card row:selected {
    background: linear-gradient(90deg, alpha(@mq_primary, 0.26), alpha(@mq_tertiary, 0.22));
}
window.mystic .card .title {
    font-size: 15px;
    font-weight: 600;
}
window.mystic .card .subtitle {
    font-size: 12px;
    color: alpha(@mq_text, 0.58);
}
window.mystic .card .calc {
    font-size: 22px;
    color: @mq_primary;
}
"#;

/// One line in the results list.
#[derive(Clone)]
enum Item {
    Calc(String),
    App(gio::AppInfo),
    File(PathBuf),
    Web(String),
}

fn main() -> glib::ExitCode {
    // Only one copy ever runs. Running `mysticquaere` again just tells the
    // running copy to show or hide itself, which is what the hotkey does.
    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::HANDLES_COMMAND_LINE)
        .build();
    app.connect_startup(build_ui);
    app.connect_command_line(|app, cmd| {
        // `mysticquaere --daemon` starts hidden (for login); anything else toggles
        let daemon = cmd.arguments().iter().any(|a| a.to_str() == Some("--daemon"));
        if !daemon {
            toggle(app);
        }
        glib::ExitCode::SUCCESS
    });
    app.run()
}

fn toggle(app: &Application) {
    if let Some(window) = app.windows().first() {
        if window.is_visible() {
            window.set_visible(false);
        } else {
            window.present();
        }
    }
}

// ---------- Apps ----------

/// Every app that would normally appear in your app menu.
fn load_apps() -> Vec<gio::AppInfo> {
    let mut apps: Vec<_> = gio::AppInfo::all()
        .into_iter()
        .filter(|a| a.should_show())
        .collect();
    apps.sort_by_key(|a| a.display_name().to_lowercase());
    apps
}

/// Lower is better; None means no match.
fn score_app(app: &gio::AppInfo, query: &str) -> Option<u8> {
    let name = app.display_name().to_lowercase();
    if name.starts_with(query) {
        Some(0)
    } else if name.split_whitespace().any(|w| w.starts_with(query)) {
        Some(1)
    } else if name.contains(query) {
        Some(2)
    } else if app
        .description()
        .is_some_and(|d| d.to_lowercase().contains(query))
    {
        Some(3)
    } else {
        None
    }
}

// ---------- Files ----------

/// Walk your home folder (skipping hidden and build folders) in the background.
fn index_home() -> Vec<PathBuf> {
    let Some(home) = glib::home_dir().to_str().map(PathBuf::from) else {
        return Vec::new();
    };
    let skip = ["node_modules", "target", "__pycache__", "venv"];
    let mut found = Vec::new();
    let mut stack = vec![home];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with('.') || skip.contains(&name.as_ref()) {
                continue;
            }
            let path = entry.path();
            if entry.file_type().is_ok_and(|t| t.is_dir()) {
                stack.push(path.clone());
            }
            found.push(path);
            if found.len() >= MAX_INDEXED_FILES {
                return found;
            }
        }
    }
    found
}

fn search_files(files: &[PathBuf], query: &str) -> Vec<PathBuf> {
    if query.len() < 2 {
        return Vec::new();
    }
    let mut hits: Vec<(u8, usize, &PathBuf)> = files
        .iter()
        .filter_map(|p| {
            let name = p.file_name()?.to_string_lossy().to_lowercase();
            let rank = if name.starts_with(query) {
                0
            } else if name.contains(query) {
                1
            } else {
                return None;
            };
            Some((rank, p.as_os_str().len(), p))
        })
        .collect();
    hits.sort_by_key(|(rank, len, _)| (*rank, *len));
    hits.into_iter().take(MAX_FILES).map(|(_, _, p)| p.clone()).collect()
}

/// "/home/you/Documents/x" becomes "~/Documents"
fn pretty_parent(path: &Path) -> String {
    let parent = path.parent().unwrap_or(path).to_string_lossy().to_string();
    let home = glib::home_dir().to_string_lossy().to_string();
    match parent.strip_prefix(&home) {
        Some(rest) => format!("~{rest}"),
        None => parent,
    }
}

// ---------- Calculator ----------

/// Tiny calculator: + - * / % ^ and brackets, e.g. "2 * (3 + 4) ^ 2".
struct Calc<'a> {
    s: &'a [u8],
    i: usize,
}

impl Calc<'_> {
    fn eval(text: &str) -> Option<f64> {
        let mut c = Calc { s: text.as_bytes(), i: 0 };
        let value = c.expr()?;
        c.space();
        (c.i == c.s.len() && value.is_finite()).then_some(value)
    }
    fn space(&mut self) {
        while self.s.get(self.i).is_some_and(|b| b.is_ascii_whitespace()) {
            self.i += 1;
        }
    }
    fn eat(&mut self, ch: u8) -> bool {
        self.space();
        if self.s.get(self.i) == Some(&ch) {
            self.i += 1;
            true
        } else {
            false
        }
    }
    fn expr(&mut self) -> Option<f64> {
        let mut v = self.term()?;
        loop {
            if self.eat(b'+') {
                v += self.term()?;
            } else if self.eat(b'-') {
                v -= self.term()?;
            } else {
                return Some(v);
            }
        }
    }
    fn term(&mut self) -> Option<f64> {
        let mut v = self.power()?;
        loop {
            if self.eat(b'*') || self.eat(b'x') {
                v *= self.power()?;
            } else if self.eat(b'/') {
                v /= self.power()?;
            } else if self.eat(b'%') {
                v %= self.power()?;
            } else {
                return Some(v);
            }
        }
    }
    fn power(&mut self) -> Option<f64> {
        let base = self.unary()?;
        if self.eat(b'^') {
            Some(base.powf(self.power()?))
        } else {
            Some(base)
        }
    }
    fn unary(&mut self) -> Option<f64> {
        if self.eat(b'-') {
            Some(-self.unary()?)
        } else {
            self.atom()
        }
    }
    fn atom(&mut self) -> Option<f64> {
        if self.eat(b'(') {
            let v = self.expr()?;
            return self.eat(b')').then_some(v);
        }
        self.space();
        let start = self.i;
        while self
            .s
            .get(self.i)
            .is_some_and(|b| b.is_ascii_digit() || *b == b'.')
        {
            self.i += 1;
        }
        std::str::from_utf8(&self.s[start..self.i]).ok()?.parse().ok()
    }
}

/// Only treat the query as maths if it has a digit and an operator.
fn calculate(query: &str) -> Option<String> {
    let has_digit = query.chars().any(|c| c.is_ascii_digit());
    let has_op = query.chars().any(|c| "+-*/%^x".contains(c));
    if !has_digit || !has_op {
        return None;
    }
    let v = Calc::eval(query)?;
    if v.fract() == 0.0 && v.abs() < 1e15 {
        Some(format!("{}", v as i64))
    } else {
        let s = format!("{v:.10}");
        Some(s.trim_end_matches('0').trim_end_matches('.').to_string())
    }
}

// ---------- Search everything ----------

fn search_all(apps: &[gio::AppInfo], files: &[PathBuf], query: &str) -> Vec<Item> {
    let raw = query.trim();
    let query = raw.to_lowercase();
    if query.is_empty() {
        return Vec::new();
    }
    let mut items = Vec::new();

    if let Some(answer) = calculate(raw) {
        items.push(Item::Calc(answer));
    }

    let mut app_hits: Vec<_> = apps
        .iter()
        .filter_map(|a| score_app(a, &query).map(|s| (s, a.clone())))
        .collect();
    app_hits.sort_by_key(|(s, _)| *s);
    items.extend(app_hits.into_iter().take(MAX_APPS).map(|(_, a)| Item::App(a)));

    items.extend(search_files(files, &query).into_iter().map(Item::File));

    items.push(Item::Web(raw.to_string()));
    items
}

// ---------- Result rows ----------

fn make_row(item: &Item) -> gtk::Box {
    let (icon, title, subtitle) = match item {
        Item::Calc(answer) => (
            Image::from_icon_name("accessories-calculator"),
            format!("= {answer}"),
            "Press Enter to copy".to_string(),
        ),
        Item::App(app) => (
            match app.icon() {
                Some(icon) => Image::from_gicon(&icon),
                None => Image::from_icon_name("application-x-executable"),
            },
            app.display_name().to_string(),
            app.description().map(|d| d.to_string()).unwrap_or_default(),
        ),
        Item::File(path) => {
            let icon = if path.is_dir() {
                Image::from_icon_name("folder")
            } else {
                let (ctype, _) = gio::content_type_guess(Some(path), None);
                Image::from_gicon(&gio::content_type_get_icon(&ctype))
            };
            let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            (icon, name, pretty_parent(path))
        }
        Item::Web(query) => (
            Image::from_icon_name("web-browser"),
            format!("Search the web for “{query}”"),
            "DuckDuckGo".to_string(),
        ),
    };
    icon.set_pixel_size(36);

    let row = gtk::Box::new(Orientation::Horizontal, 12);
    row.set_margin_top(4);
    row.set_margin_bottom(4);

    let title_label = Label::new(Some(&title));
    title_label.set_xalign(0.0);
    title_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    title_label.add_css_class("title");
    if matches!(item, Item::Calc(_)) {
        title_label.add_css_class("calc");
    }

    let text = gtk::Box::new(Orientation::Vertical, 2);
    text.set_valign(gtk::Align::Center);
    text.append(&title_label);
    if !subtitle.is_empty() {
        let sub = Label::new(Some(&subtitle));
        sub.set_xalign(0.0);
        sub.set_ellipsize(gtk::pango::EllipsizeMode::End);
        sub.add_css_class("subtitle");
        text.append(&sub);
    }

    row.append(&icon);
    row.append(&text);
    row
}

/// Do whatever the chosen result is for, then hide.
fn activate(item: &Item, window: &ApplicationWindow) {
    let display = WidgetExt::display(window);
    let context = display.app_launch_context();
    let result = match item {
        Item::Calc(answer) => {
            display.clipboard().set_text(answer);
            Ok(())
        }
        Item::App(app) => app.launch(&[], Some(&context)),
        Item::File(path) => {
            let uri = gio::File::for_path(path).uri();
            gio::AppInfo::launch_default_for_uri(&uri, Some(&context))
        }
        Item::Web(query) => {
            let q = glib::Uri::escape_string(query, None::<&str>, false);
            let url = format!("https://duckduckgo.com/?q={q}");
            gio::AppInfo::launch_default_for_uri(&url, Some(&context))
        }
    };
    if let Err(err) = result {
        eprintln!("MysticQuaere: {err}");
    }
    window.set_visible(false);
}

// ---------- Adaptive theme ----------

type Rgb = (u8, u8, u8);

/// The handful of colours the launcher needs, picked from your wallpaper.
struct Palette {
    surface: Rgb,
    text: Rgb,
    primary: Rgb,
    tertiary: Rgb,
}

impl Palette {
    /// Obsidian Veil, used when no wallpaper colours can be found.
    fn fallback() -> Self {
        Palette {
            surface: (20, 16, 30),
            text: (237, 232, 255),
            primary: (185, 164, 255),
            tertiary: (255, 170, 220),
        }
    }

    fn is_dark(&self) -> bool {
        let (r, g, b) = self.surface;
        let luma = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
        luma < 128.0
    }
}

fn config_home() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| glib::home_dir().join(".config"))
}

/// Where wallpaper tools write their colours, best first:
/// matugen in end-4's dots (switchwall.sh), then pywal.
fn palette_files() -> Vec<PathBuf> {
    let home = glib::home_dir();
    let state = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".local/state"));
    let cache = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".cache"));
    vec![
        state.join("quickshell/user/generated/colors.json"),
        cache.join("wal/colors.json"),
    ]
}

/// Your own tweaks, layered on top of everything: ~/.config/mysticquaere/style.css
fn user_css_file() -> PathBuf {
    config_home().join("mysticquaere/style.css")
}

/// Finds `"key": "#rrggbb"` in a colours file. Both files are flat enough
/// that this avoids pulling in a JSON library.
fn hex_for(text: &str, key: &str) -> Option<Rgb> {
    let at = text.find(&format!("\"{key}\""))? + key.len() + 2;
    let rest = text[at..].trim_start().strip_prefix(':')?.trim_start();
    let hex = rest.strip_prefix('"')?.strip_prefix('#')?.get(..6)?;
    let n = u32::from_str_radix(hex, 16).ok()?;
    Some(((n >> 16) as u8, (n >> 8) as u8, n as u8))
}

fn read_palette() -> Palette {
    for path in palette_files() {
        let Ok(text) = std::fs::read_to_string(&path) else { continue };
        // matugen (Material You) names
        if let (Some(surface), Some(text_c), Some(primary), Some(tertiary)) = (
            hex_for(&text, "surface_container_low").or(hex_for(&text, "surface")),
            hex_for(&text, "on_surface"),
            hex_for(&text, "primary"),
            hex_for(&text, "tertiary"),
        ) {
            return Palette { surface, text: text_c, primary, tertiary };
        }
        // pywal names
        if let (Some(surface), Some(text_c), Some(primary), Some(tertiary)) = (
            hex_for(&text, "background"),
            hex_for(&text, "foreground"),
            hex_for(&text, "color4"),
            hex_for(&text, "color5"),
        ) {
            return Palette { surface, text: text_c, primary, tertiary };
        }
    }
    Palette::fallback()
}

fn palette_css() -> String {
    let p = read_palette();
    let hex = |(r, g, b): Rgb| format!("#{r:02x}{g:02x}{b:02x}");
    let alpha = if p.is_dark() { "0.60" } else { "0.72" };
    format!(
        "@define-color mq_surface {};\n@define-color mq_text {};\n\
         @define-color mq_primary {};\n@define-color mq_tertiary {};\n{}",
        hex(p.surface),
        hex(p.text),
        hex(p.primary),
        hex(p.tertiary),
        CSS.replace("CARD_ALPHA", alpha),
    )
}

/// Loads the wallpaper theme and your style.css, and reloads them whenever
/// the wallpaper changes or you save style.css.
fn setup_theme(display: &gtk::gdk::Display) {
    // Above the user's gtk.css (which matugen rewrites on every wallpaper
    // change), so other themes can't repaint the launcher.
    let theme = gtk::CssProvider::new();
    theme.load_from_data(&palette_css());
    gtk::style_context_add_provider_for_display(
        display,
        &theme,
        gtk::STYLE_PROVIDER_PRIORITY_USER + 1,
    );

    let user = gtk::CssProvider::new();
    let load_user = {
        let user = user.clone();
        move || {
            let text = std::fs::read_to_string(user_css_file()).unwrap_or_default();
            user.load_from_data(&text);
        }
    };
    load_user();
    gtk::style_context_add_provider_for_display(
        display,
        &user,
        gtk::STYLE_PROVIDER_PRIORITY_USER + 2,
    );

    for path in palette_files() {
        let theme = theme.clone();
        watch(&path, move || theme.load_from_data(&palette_css()));
    }
    watch(&user_css_file(), load_user);
}

/// Calls `reload` whenever the file is written, created or replaced.
fn watch(path: &Path, reload: impl Fn() + 'static) {
    let Ok(monitor) =
        gio::File::for_path(path).monitor_file(gio::FileMonitorFlags::WATCH_MOVES, gio::Cancellable::NONE)
    else {
        return;
    };
    monitor.connect_changed(move |_, _, _, event| {
        use gio::FileMonitorEvent as E;
        if matches!(event, E::ChangesDoneHint | E::Created | E::MovedIn | E::Renamed) {
            reload();
        }
    });
    std::mem::forget(monitor); // keep watching for the app's lifetime
}

// ---------- Window ----------

fn build_ui(app: &Application) {
    // Stay running in the background even while the window is hidden
    std::mem::forget(app.hold());

    // App list, refreshed automatically when you install or remove apps
    let apps = Rc::new(RefCell::new(load_apps()));
    {
        let apps = apps.clone();
        let monitor = gio::AppInfoMonitor::get();
        monitor.connect_changed(move |_| {
            *apps.borrow_mut() = load_apps();
        });
        std::mem::forget(monitor); // keep watching for the app's lifetime
    }

    // File index, rebuilt in the background each time the launcher opens
    let files: Arc<Mutex<Vec<PathBuf>>> = Arc::new(Mutex::new(Vec::new()));
    let indexing = Arc::new(AtomicBool::new(false));
    let reindex = {
        let files = files.clone();
        move || {
            if indexing.swap(true, Ordering::SeqCst) {
                return; // already running
            }
            let files = files.clone();
            let indexing = indexing.clone();
            std::thread::spawn(move || {
                let fresh = index_home();
                *files.lock().unwrap() = fresh;
                indexing.store(false, Ordering::SeqCst);
            });
        }
    };
    reindex();

    let results: Rc<RefCell<Vec<Item>>> = Rc::new(RefCell::new(Vec::new()));

    let entry = SearchEntry::builder()
        .placeholder_text("Seek apps, files, maths…")
        .hexpand(true)
        .build();

    let list = ListBox::new();
    list.set_visible(false);
    list.set_can_focus(false);

    let separator = gtk::Separator::new(Orientation::Horizontal);
    separator.set_visible(false);

    let layout = gtk::Box::new(Orientation::Vertical, 0);
    layout.add_css_class("card");
    layout.append(&entry);
    layout.append(&separator);
    layout.append(&list);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("MysticQuaere")
        .default_width(680)
        .decorated(false)
        .resizable(false)
        .child(&layout)
        .css_classes(["mystic"])
        .build();

    // Colours follow your wallpaper and update live when it changes
    setup_theme(&WidgetExt::display(&window));

    // Float above everything and take the keyboard, like Spotlight.
    // Works on KDE, Hyprland, Sway and most Wayland desktops. GNOME doesn't
    // support this, so there it falls back to a normal window.
    let wayland = std::env::var_os("WAYLAND_DISPLAY").is_some();
    if wayland && gtk4_layer_shell::is_supported() {
        window.init_layer_shell();
        // "launcher" matches your Hyprland blur rule, giving the frosted look
        window.set_namespace(Some("launcher"));
        window.set_layer(Layer::Overlay);
        window.set_keyboard_mode(KeyboardMode::Exclusive);
        window.set_anchor(Edge::Top, true);
        window.set_margin(Edge::Top, 200);
    } else {
        // Normal-window fallback (GNOME): hide when you click elsewhere
        window.connect_is_active_notify(|window| {
            if !window.is_active() {
                window.set_visible(false);
            }
        });
    }

    // Start fresh every time the launcher appears
    {
        let entry = entry.clone();
        window.connect_visible_notify(move |window| {
            if window.is_visible() {
                entry.set_text("");
                entry.grab_focus();
                reindex();
            }
        });
    }

    // Typing: refresh the results list
    {
        let list = list.clone();
        let separator = separator.clone();
        let results = results.clone();
        entry.connect_changed(move |entry| {
            while let Some(child) = list.first_child() {
                list.remove(&child);
            }
            let found = search_all(&apps.borrow(), &files.lock().unwrap(), &entry.text());
            for item in &found {
                list.append(&make_row(item));
            }
            list.set_visible(!found.is_empty());
            separator.set_visible(!found.is_empty());
            list.select_row(list.row_at_index(0).as_ref());
            *results.borrow_mut() = found;
        });
    }

    // Enter: use the highlighted result
    {
        let list = list.clone();
        let results = results.clone();
        let window = window.clone();
        entry.connect_activate(move |_| {
            let index = list.selected_row().map_or(0, |r| r.index() as usize);
            let item = results.borrow().get(index).cloned();
            if let Some(item) = item {
                activate(&item, &window);
            }
        });
    }

    // Clicking a result also uses it
    {
        let results = results.clone();
        let window = window.clone();
        list.connect_row_activated(move |_, row| {
            let item = results.borrow().get(row.index() as usize).cloned();
            if let Some(item) = item {
                activate(&item, &window);
            }
        });
    }

    // Up/Down move the highlight, Escape hides
    let keys = gtk::EventControllerKey::new();
    keys.set_propagation_phase(gtk::PropagationPhase::Capture);
    {
        let window = window.clone();
        let list = list.clone();
        keys.connect_key_pressed(move |_, key, _, _| {
            let current = list.selected_row().map_or(0, |r| r.index());
            let target = match key {
                gtk::gdk::Key::Escape => {
                    window.set_visible(false);
                    return glib::Propagation::Stop;
                }
                gtk::gdk::Key::Down => current + 1,
                gtk::gdk::Key::Up => current - 1,
                _ => return glib::Propagation::Proceed,
            };
            if let Some(row) = list.row_at_index(target) {
                list.select_row(Some(&row));
            }
            glib::Propagation::Stop
        });
    }
    window.add_controller(keys);

    // Typing anywhere in the window goes into the search bar
    entry.set_key_capture_widget(Some(&window));
}
