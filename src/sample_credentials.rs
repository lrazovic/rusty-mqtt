pub struct Credentials<'a> {
    pub appid: &'a str,
    pub appaccesskey: &'a str,
}

impl Credentials<'static> {
    pub const fn new(appid: &'static str, appaccesskey: &'static str) -> Self {
        Self {
            appid,
            appaccesskey,
        }
    }
}

pub const fn get() -> Credentials<'static> {
    // Application ID from the TTN Console
    let appid = "app_name";
    // App Access API key from the TTN Console
    let appaccesskey = "app_access_key";
    Credentials::new(appid, appaccesskey)
}
