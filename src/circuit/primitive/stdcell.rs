use reda_lib::model::LibCell;
use reda_sp::Subckt;
use crate::circuit::{Design, Shr, Port};

use super::Primitive;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum StdcellKind {
    Inv,
    And(usize),
    Or(usize),
    Nand(usize),
    Nor(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DriveStrength {
    X1, X2, X4, X8, X16, X32,
}

pub struct Stdcell {
    pub name: String,
    pub drive_strength: DriveStrength,
    pub kind: StdcellKind,
    pub ports: Vec<Shr<Port>>,
    pub netlist: Subckt,
}

impl DriveStrength {
    pub fn try_from_cell(cell: &LibCell) -> Option<Self> {
        let name = &cell.name.to_lowercase();
        if name.contains("x32") {
            return Some(Self::X32);
        }
        if name.contains("x16") {
            return Some(Self::X16);
        }
        if name.contains("x8") {
            return Some(Self::X8);
        }
        if name.contains("x4") {
            return Some(Self::X4);
        }
        if name.contains("x2") {
            return Some(Self::X2);
        }
        if name.contains("x1") {
            return Some(Self::X1);
        }
        None
    }
}

impl Design for Stdcell {
    fn ports(&self) -> &[Shr<Port>] {
        &self.ports
    }
}

impl Primitive for Stdcell {
    fn netlist(&self) -> &Subckt {
        &self.netlist
    }
}