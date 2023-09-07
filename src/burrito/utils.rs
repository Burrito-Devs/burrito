
pub fn get_home_dir() -> String {
    home::home_dir().unwrap_or("/root".into()).to_string_lossy().into_owned()
}

pub fn get_burrito_dir() -> String {
    let mut s = get_home_dir();
    s.push_str("/.burrito/");
    s
}
