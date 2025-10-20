mod leafcell;
mod stdcell;

pub use leafcell::*;
pub use stdcell::*;
use reda_sp::Subckt;
use super::{Design, Shr};

pub trait Primitive : Design {
    fn netlist(&self) -> &Subckt;
}

pub type ShrPrimitive = Shr<Box<dyn Primitive>>;

// #[derive(Hash, PartialEq, Eq)]
// pub enum ShrPrimitive {
//     LogicGate(Shr<LogicGate>),
//     Dff(Shr<Dff>),
//     Leafcell(Shr<Leafcell>),
// }

// impl Into<ShrPrimitive> for Shr<LogicGate> {
//     fn into(self) -> ShrPrimitive {
//         ShrPrimitive::LogicGate(self)
//     }
// }

// impl Into<ShrPrimitive> for Shr<Dff> {
//     fn into(self) -> ShrPrimitive {
//         ShrPrimitive::Dff(self)
//     }
// }

// impl Into<ShrPrimitive> for Shr<Leafcell> {
//     fn into(self) -> ShrPrimitive {
//         ShrPrimitive::Leafcell(self)
//     }
// }

// impl ShrPrimitive {
//     pub fn name(&self) -> ShrString {
//         match self {
//             Self::LogicGate(p) => p.read().name(),
//             Self::Dff(p) => p.read().name(),
//             Self::Leafcell(p) => p.read().name(),
//         }
//     }

//     pub fn ports(&self) -> MappedRwLockReadGuard<'_, [Shr<Port>]> {
//         match self {
//             Self::LogicGate(p) => RwLockReadGuard::map(p.read(), |p| p.ports()),
//             Self::Dff(p) => RwLockReadGuard::map(p.read(), |p| p.ports()),
//             Self::Leafcell(p) => RwLockReadGuard::map(p.read(), |p| p.ports()),
//         }
//     }

//     pub fn netlist(&self) -> MappedRwLockReadGuard<'_, Subckt> {
//         match self {
//             Self::LogicGate(p) => RwLockReadGuard::map(p.read(), |p| p.netlist()),
//             Self::Dff(p) => RwLockReadGuard::map(p.read(), |p| p.netlist()),
//             Self::Leafcell(p) => RwLockReadGuard::map(p.read(), |p| p.netlist()),
//         }
//     }
// }