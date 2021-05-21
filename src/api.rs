use chrono::prelude::*;
use reqwest::{self, IntoUrl};
use serde::Deserialize;
use std::collections::HashMap;

use crate::bout::Bout;
use crate::response::Response;

#[derive(Deserialize, Debug)]
struct ApiBoutResult {
    code: String,
    result: JBout,
}

#[derive(Deserialize, Debug)]
struct JBout {
    id: usize,
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

#[derive(Deserialize, Debug)]
struct ApiTournamentResult {
    code: String,
    result: JContent,
}

#[derive(Deserialize, Debug)]
struct JContent {
    content: Vec<JBout>,
}

fn parse_tournament_data(data: &str) -> serde_json::Result<ApiTournamentResult> {
    let parsed: ApiTournamentResult = serde_json::from_str(&data)?;
    Ok(parsed)
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
async fn get_bout(bout_id: usize) -> Result<Bout, Response> {
    let address = format!("https://api.spire.gg/matches/{}", bout_id);

    match make_request(&address).await {
        Ok(data) => match parse_bout_data(&data) {
            Ok(parsed) => {
                let tournament_name = parsed.result.tournament.name;
                let mut raw_dt = parsed.result.datetime.clone();
                raw_dt.push('Z');

                let datetime = raw_dt.parse::<DateTime<Utc>>().unwrap();
                let maps = parsed
                    .result
                    .maps
                    .into_iter()
                    .map(|jmap| jmap.name)
                    .collect();

                let home = parsed.result.lineups.get(&'A').unwrap().name.clone();
                let away = parsed.result.lineups.get(&'B').unwrap().name.clone();

                let bout = Bout::new(bout_id, tournament_name, datetime, maps, home, away);
                Ok(bout)
            }
            Err(why) => Err(create_api_error_response(why.to_string(), address)),
        },
        Err(why) => Err(create_api_error_response(why.to_string(), address)),
    }
}

pub async fn find_next_bout(tournament_id: usize, team_id: usize) -> Result<Bout, Response> {
    let address = format!(
        "https://api.spire.gg/matches?tournamentId={}",
        tournament_id
    );
    match make_request(&address).await {
        Ok(data) => match parse_tournament_data(&data) {
            Ok(parsed) => {
                let team_bouts: Vec<_> = parsed
                    .result
                    .content
                    .iter()
                    .filter(|jbout| {
                        jbout.lineups.get(&'A').unwrap().id == team_id
                            || jbout.lineups.get(&'B').unwrap().id == team_id
                    })
                    .collect();

                if team_bouts.len() == 0 {
                    let title = String::from("No active matches found");
                    let message = format!(
                        "For further information see https://spire.gg/tournament/{}#brackets.",
                        tournament_id
                    );
                    let response = Response::new_error(title, message);
                    return Err(response);
                }

                let bout_id = team_bouts[0].id;
                get_bout(bout_id).await
            }

            Err(why) => Err(create_api_error_response(why.to_string(), address)),
        },
        Err(why) => Err(create_api_error_response(why.to_string(), address)),
    }
}

fn create_api_error_response(why: String, address: String) -> Response {
    let title = String::from("API error");
    let message = format!("Error parsing response of \"{}\"!\n\t{}", address, why);
    Response::new_error(title, message)
}
