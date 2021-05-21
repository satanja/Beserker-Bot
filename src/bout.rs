pub type Player = String;
pub type MapName = String;
use crate::response::Response;
use chrono::prelude::*;

#[derive(Debug)]
pub struct Bout {
    id: usize,
    tournament: String,
    datetime: DateTime<Local>,
    maps: Vec<(MapName, Option<Player>)>,
    home: String,
    away: String,
}

impl Eq for Bout {}

impl PartialEq for Bout {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Bout {
    pub fn new(
        id: usize,
        tournament: String,
        utc_datetime: DateTime<Utc>,
        maps: Vec<String>,
        home: String,
        away: String,
    ) -> Bout {
        let datetime = utc_datetime.with_timezone(&Local);
        let maps = maps.into_iter().map(|map| (map, None)).collect();
        Bout {
            id,
            tournament,
            datetime,
            maps,
            home,
            away,
        }
    }

    fn is_valid_index(&self, index: usize) -> Result<(), Response> {
        if index >= self.maps.len() || index == 0 {
            let text = format!("Please enter a number between 1 and {}", self.maps.len() - 1);
            let response = Response::new_error(String::from("Invalid index."), text);
            return Err(response);
        }

        Ok(())
    }

    pub fn insert_player(&mut self, index: usize, player: String) -> Result<(), Response> {
        self.is_valid_index(index)?;

        self.maps[index - 1].1 = Some(player);
        Ok(())
    }

    pub fn remove_player(&mut self, index: usize) -> Result<(), Response> {
        self.is_valid_index(index)?;

        self.maps[index - 1].1 = None;
        Ok(())
    }

    pub fn get_title(&self) -> String {
        format!("{} vs {}", &self.home, &self.away)
    }

    pub fn get_description(&self) -> String {
        let date = self.datetime.format("%A %B %d, %Y").to_string();
        let time = self.datetime.time().to_string();
        let remaining = self.datetime.signed_duration_since(Local::now());
        let days = remaining.num_days();
        let hours = remaining.num_hours() - days * 24;
        let min = remaining.num_minutes() - days * 24 * 60 - hours * 60;

        let url = format!("https://spire.gg/match/{}", self.id);
        format!(
            "Date: {}\nTime: {} (in: {}d {}hr {}min)\nChannel: spire{}\n{}",
            date, time, days, hours, min, self.id, url
        )
    }

    pub fn get_maps(&self) -> String {
        let mut result = String::new();
        let maps = self.maps.len();

        for i in 0..maps {
            let map = &self.maps[i];
            if let Some(name) = &map.1 {
                result.push_str(name);
            } else {
                result.push('[');
                if i < maps - 1 {
                    result.push_str(&(i + 1).to_string());
                } else {
                    result.push_str("ACE")
                }
                result.push(']');
            }
            result.push_str(": ");
            result.push_str(&map.0);
            result.push('\n');
        }
        result
    }
}
