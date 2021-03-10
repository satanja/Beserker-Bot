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

/// Gets the match (referred to as Bout to avoid overlap with the Rust keyword
/// `match`) with `bout_id` from the spire.gg API.
async fn get_bout(bout_id: usize) -> Result<Bout, String> {
    let address = format!("https://api.spire.gg/matches/{}", bout_id);

    match make_request(&address).await {
        Ok(data) => match parse_bout_data(&data) {
            Ok(parsed) => {
                let tournament_name = parsed.result.tournament.name;
                let datetime = parsed.result.datetime;
                let maps = parsed
                    .result
                    .maps
                    .into_iter()
                    .map(|jmap| jmap.name)
                    .collect();

                let bout = Bout::new(bout_id, tournament_name, datetime, maps);
                Ok(bout)
            }
            Err(why) => Err(format!(
                "Error parsing response of \"{}\"!\n\t{}",
                address, why
            )),
        },
        Err(why) => Err(format!("api endpoint error at \"{}\"!\n\t{}", address, why)),
    }
}


pub async fn find_next_bout(tournament_id: usize, team_id: usize) -> Result<Bout, String> {
    get_bout(2156).await
}