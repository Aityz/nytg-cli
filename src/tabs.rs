#[derive(serde::Serialize, serde::Deserialize)]
pub struct Tabber {
    pub index: u8,
    pub values: Vec<String>,
}

impl Tabber {
    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.values.len() as u8;
    }

    pub fn prev(&mut self) {
        if self.index != 0 {
            self.index -= 1;
        } else {
            self.index = (self.values.len() - 1) as u8
        }
    }
}
