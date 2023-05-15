use base64::{engine::general_purpose, Engine as _};
use serde::Deserialize;
use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;
use wmi::{COMLibrary, FilterValue, WMIConnection};

#[derive(Deserialize, Debug)]
#[allow(non_camel_case_types, non_snake_case, dead_code)]
struct Win32_Process {
    Name: String,
    ProcessId: u32,
    CommandLine: String,
}

struct UserInfo {

}

fn main() -> anyhow::Result<()> {
    let com_con = COMLibrary::new()?;
    let wmi_con = WMIConnection::new(com_con.into())?;

    let mut filters = HashMap::new();
    filters.insert("Name".to_owned(), FilterValue::Str("LeagueClientUx.exe"));

    let results = wmi_con.filtered_query::<Win32_Process>(&filters)?;
    let args: HashMap<&str, &str> = results
        .get(0)
        .unwrap()
        .CommandLine
        .split(" \"")
        .skip(1)
        .filter(|x| x.contains("riotclient-auth-token") || x.contains("riotclient-app-port"))
        .map(|x| {
            let formatted = x.get(2..x.len() - 1).unwrap();
            let (first, last) = formatted.split_at(formatted.find("=").unwrap());
            (first, last.get(1..).unwrap())
        })
        .collect();
    dbg!(&args);
    let auth_token = args.get("riotclient-auth-token").unwrap();
    let auth_token = format!("riot:{}", auth_token);
    let auth_token = general_purpose::URL_SAFE_NO_PAD.encode(auth_token.as_bytes());
    let app_port = args.get("riotclient-app-port").unwrap();

    //Loop to follow
    //see if user is logged -> /rso-auth/v1/authorization/userinfo
    //if the user is logged in await game to start
    //to see this /chat/v5/participants/champ-select should have 5 users
    //send link with op.gg
    //once participants is empty, clear the screen

    let request_base = format!("https://127.0.0.1:{}/", app_port);

    let mut headers = reqwest::header::HeaderMap::new();

    headers.insert(
        "Authorization",
        reqwest::header::HeaderValue::from_str(&auth_token).unwrap(),
    );
    headers.insert(
        "User-Agent",
        reqwest::header::HeaderValue::from_str("LeagueOfLegendsClient").unwrap(),
    );
    headers.insert(
        "Accep",
        reqwest::header::HeaderValue::from_str("application/json").unwrap(),
    );
    let client = reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true)
        .default_headers(headers.clone())
        .build()?;

    loop {
        sleep(Duration::from_secs(2));

        let user_info = client
            .get(format!(
                "{}{}",
                request_base, "rso-auth/v1/authorization/userinfo"
            ))
            .send()?
            .text()?;
    }
}
