use reqwest::{self, IntoUrl};
use serde::Deserialize;
use std::collections::HashMap;

use crate::bout::Bout;

#[derive(Deserialize, Debug)]
struct ApiBoutResult {
    code: String,
    result: JBout,
}

#[derive(Deserialize, Debug)]
struct JBout {
    datetime: String,
    maps: Vec<JMap>,
    tournament: JTournament,
    lineups: HashMap<char, JTeam>,
}
#[derive(Deserialize, Debug)]
struct JMap {
    id: usize,
    name: String,
}

#[derive(Deserialize, Debug)]
struct JTournament {
    id: usize,
    name: String,
}

#[derive(Deserialize, Debug)]
struct JTeam {
    id: usize,
    name: String,
}

/// Attempt to parse raw JSON bout data to a `ApiBoutResult`. 
fn parse_bout_data(data: &str) -> serde_json::Result<ApiBoutResult> {
    let parsed: ApiBoutResult = serde_json::from_str(&data)?;
    Ok(parsed)
}

/// Posts a GET-request to a specific URL.
async fn make_request<T: IntoUrl>(request: T) -> Result<String, reqwest::Error> {
    let result = reqwest::get(request).await?.text().await?;
    Ok(result)
}

/// Gets the latest match (referred to as Bout to avoid overlap with the Rust keyword `match`) 
/// given a `bout_id` from the spire.gg API.
pub async fn get_bout(bout_id: usize, old_bout: Option<Bout>) -> Result<Bout, String> {
    // TODO: use actual old_bout

    let address = format!("https://api.spire.gg/matches/{}", bout_id);

    match make_request(&address).await {
        Ok(data) => {
            match parse_bout_data(&data) {
                Ok(parsed) => {
                    println!("{:?}", parsed);

                    // TODO:
                    // verify whether the parsed bout is new
                    // if not return the old bout,
                    // if the parsed bout is new, construct a new Bout
                    unimplemented!("implement get_bout logic");
                }
                Err(why) => Err(format!(
                    "Error parsing resposne of \"{}\"!\n\t{}",
                    address, why
                )),
            }
        }
        _ => Err("api endpoint error!".to_string()),
    }
}
