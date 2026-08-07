use crate::planet::traits::Planet;

pub struct Kerbin {
    name: &'static str
}

impl Planet for Kerbin {
    fn name(&self) -> &'static str {
        "Kerbin"
    }
}