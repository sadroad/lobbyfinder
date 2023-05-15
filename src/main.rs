use serde::Deserialize;
use std::collections::HashMap;
use wmi::{COMLibrary, FilterValue, WMIConnection};

#[derive(Deserialize, Debug)]
#[allow(non_camel_case_types,non_snake_case, dead_code)]
struct Win32_Process {
    Name: String,
    ProcessId: u32,
    CommandLine: String,
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
    let app_port = args.get("riotclient-app-port").unwrap();

    Ok(())
}
