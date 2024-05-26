// Generic Trait for collecting different data
pub trait DataCollector {
    type Output;
    type Params;
    fn collect(&self, params: Self::Params) -> Self::Output;
}

pub mod cpu;