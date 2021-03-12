pub type Player = String;
pub type MapName = String;
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
    pub fn new(id: usize, tournament: String, utc_datetime: DateTime<Utc>, maps: Vec<String>, home: String, away: String) -> Bout {
        let datetime = utc_datetime.with_timezone(&Local);
        let maps = maps.into_iter().map(|map| (map, None)).collect();
        Bout {
            id,
            tournament,
            datetime,
            maps,
            home,
            away
        }
    }

    pub fn insert_player(&mut self, index: usize, player: String) -> Result<(), String> {
        if index >= 5 || index == 0 {
            return Err(format!("Index out of bounds: {}", index));
        }
        
        self.maps[index - 1].1 = Some(player);

        Ok(())
    }

    pub fn remove_player(&mut self, index: usize) -> Result<(), String> {
        if index >= 5 || index == 0 {
            return Err(format!("Index out of bounds: {}", index));
        }
        self.maps[index - 1].1 = None;

        Ok(())
    }

    fn get_tournament(&self) -> &str {
        &self.tournament
    }

    pub fn get_title(&self) -> String {
        format!("{} vs {}", &self.home, &self.away)
    }

    pub fn get_description(&self) -> String {
        let date = self.datetime.format("%A %B %d, %Y").to_string();
        let time = self.datetime.time().to_string();
        let url = format!("https://spire.gg/match/{}", self.id);
        format!("Date: {}\nTime: {}\nChannel: spire{}\n{}", date, time, self.id, url)
    }

    pub fn get_maps(&self) -> String {
        let mut result = String::new();
        for i in 0..5 {
            let map = &self.maps[i];
            if let Some(name) = &map.1 {
                result.push_str(name);
            } else {
                result.push('[');
                match i {
                    0 => result.push('1'),
                    1 => result.push('2'),
                    2 => result.push('3'),
                    3 => result.push('4'),
                    4 => {
                        result.push_str("ACE");
                    }
                    _ => panic!("does not happen"),
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
