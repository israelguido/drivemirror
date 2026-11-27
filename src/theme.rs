use gtk::gdk;
use std::cell::RefCell;

thread_local! {
    static PROVIDER: RefCell<Option<(String, gtk::CssProvider)>> = const { RefCell::new(None) };
}

pub fn apply() {
    let colors = directories::BaseDirs::new().map(|dirs| {
        dirs.home_dir()
            .join(".local/state/omarchy/current/theme/colors.toml")
    });
    let accent = colors
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|text| {
            text.lines().find_map(|line| {
                let (key, value) = line.split_once('=')?;
                (key.trim() == "accent").then(|| value.trim().trim_matches(['\'', '"']).to_string())
            })
        })
        .filter(|value| value.starts_with('#') && value.len() == 7)
        .unwrap_or_else(|| "#3584e4".into());
    PROVIDER.with(|cell| {
        let mut current = cell.borrow_mut();
        if current.as_ref().is_some_and(|(value, _)| value == &accent) {
            return;
        }
        let provider = current
            .as_ref()
            .map(|(_, provider)| provider.clone())
            .unwrap_or_default();
        provider.load_from_data(&format!(".sync-button {{ background: {accent}; color: #fff; font-weight: 700; }} .card {{ padding: 18px; }}"));
        if current.is_none()
            && let Some(display) = gdk::Display::default()
        {
            gtk::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
        *current = Some((accent, provider));
    });
}
