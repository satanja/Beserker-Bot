pub type Player = String;
pub type MapName = String;
// use chrono::DateTime;

#[derive(Debug)]
pub struct Bout {
    id: usize,
    tournament: String,
    datetime: String,
    maps: Vec<(MapName, Option<Player>)>,
    // streams: Vec<String>,
}

impl Eq for Bout {}

impl PartialEq for Bout {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Bout {
    pub fn new(id: usize, tournament: String, datetime: String, maps: Vec<String>) -> Bout {
        let maps = maps.into_iter().map(|map| (map, None)).collect();
        Bout {
            id,
            tournament,
            datetime,
            maps,
        }
    }

    pub fn insert_player(&mut self, index: usize, player: String) {
        self.maps[index].1 = Some(player);
    }

    pub fn remove_player(&mut self, index: usize) {
        self.maps[index].1 = None;
    }

    pub fn get_tournament(&self) -> &str {
        &self.tournament
    }

    pub fn get_date(&self) -> &str {
        &self.datetime
    }

    pub fn get_maps(&self) -> &Vec<(MapName, Option<Player>)> {
        &self.maps
    }

    pub fn get_channel(&self) -> String {
        format!("spire{}", self.id)
    }
}
