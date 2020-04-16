pub struct Credentials {
    pub appid: String,
    pub appaccesskey: String,
}

impl Credentials {
    pub fn new(appid: String, appaccesskey: String) -> Self {
        Self {
            appid,
            appaccesskey,
        }
    }
}

pub fn get_credentials() -> Credentials {
    //App ID from the TTN Console
    let appid = String::from("XXXXXXXX");
    //App Access Key from the TTN Console
    let appaccesskey = String::from("ttn-account-v2.XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");
    Credentials::new(appid, appaccesskey)
}
