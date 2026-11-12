use std::error::Error;

pub struct Pipeline {}

impl Pipeline {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {})
    }
}
