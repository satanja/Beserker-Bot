pub type Player = String;
pub type MapName = String;

#[derive(Debug)]
pub struct Bout {
    tournament: String,
    date: usize,
    maps: Vec<(MapName, Option<Player>)>,
}

impl Eq for Bout {}

impl PartialEq for Bout {
    fn eq(&self, other: &Self) -> bool {
        self.date == other.date        
    }
}

impl Bout {

    pub fn new(tournament: String, date: usize, maps: Vec<String>) -> Bout {
        let maps = maps.into_iter().map(|map| (map, None)).collect();
        Bout { tournament, date, maps }
    }

    pub fn set_player(&mut self, index: usize, player: String) {
        self.maps[index].1 = Some(player);
    }

    pub fn remove_player(&mut self, index: usize) {
        self.maps[index].1 = None;
    }

    pub fn get_tournament(&self) -> &str {
        &self.tournament
    }

    pub fn get_date(&self) -> &usize {
        &self.date
    }

    pub fn get_maps(&self) -> &Vec<(MapName, Option<Player>)> {
        &self.maps
    }
}