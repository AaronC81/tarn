use super::BodySection;

pub struct FunctionSection {
    pub types: Vec<u32>,
}

impl BodySection for FunctionSection {
    const ID: u8 = 3;
    type BodyItem = u32;
    fn body_item(&self) -> &Vec<Self::BodyItem> { &self.types }
}
