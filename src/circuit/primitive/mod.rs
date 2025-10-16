mod leafcell;
mod stdcell;

pub use leafcell::*;
pub use stdcell::*;
use reda_sp::Subckt;
use super::Design;

pub trait Primitive : Design {
    fn netlist(&self) -> &Subckt;
}
