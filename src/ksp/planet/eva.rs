use crate::ksp::planet::traits::Planet;

pub struct Eva {
    name: &'static str
}

impl Planet for Eva {
    fn name(&self) -> &'static str {
        "Eva"
    }
}