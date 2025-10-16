use std::fmt::Display;

use reda_lib::model::LibCell;
use reda_sp::Subckt;
use crate::circuit::{CircuitError, Design, Port, Shr, ShrString};

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

#[derive(Debug)]
pub struct Stdcell {
    pub name: ShrString,
    pub drive_strength: DriveStrength,
    pub kind: StdcellKind,
    pub ports: Vec<Shr<Port>>,
    pub netlist: Subckt,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum StdcellPort {
    Input(usize),
    Output,
    Vdd,
    Gnd,
}

impl Stdcell {
    pub fn input_ports(&self) -> impl Iterator<Item = &Shr<Port>> {
        self.ports.iter()
            .filter(|port| port.read().is_input())
    }

    pub fn output_ports(&self) -> impl Iterator<Item = &Shr<Port>> {
        self.ports.iter()
            .filter(|port| port.read().is_output())
    }

    pub fn source_ports(&self) -> impl Iterator<Item = &Shr<Port>> {
        self.ports.iter()
            .filter(|port| port.read().is_source())
    }

    pub fn input_pn(&self, order: usize) -> Result<ShrString, CircuitError> {
        self.input_ports()
            .nth(order)
            .ok_or_else(|| CircuitError::StdcellInputPortOutOfRange(order))
            .map(|port| port.read().name.clone())
    }

    pub fn output_pn(&self) -> ShrString {
        self.output_ports()
            .nth(0)
            .map(|port| port.read().name.clone())
            .unwrap()
    }

    // TODO
    pub fn vdd_pn(&self) -> ShrString {
        self.source_ports()
            .nth(0)
            .map(|port| port.read().name.clone())
            .unwrap()
    }

    pub fn gnd_pn(&self) -> ShrString {
        self.source_ports()
            .nth(1)
            .map(|port| port.read().name.clone())
            .unwrap()
    }
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

impl Display for DriveStrength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::X1  => write!(f, "x1"),
            Self::X2  => write!(f, "x2"),
            Self::X4  => write!(f, "x4"),
            Self::X8  => write!(f, "x8"),
            Self::X16 => write!(f, "x16"),
            Self::X32 => write!(f, "x32"),
        }
    }
}

impl Display for StdcellKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Inv => write!(f, "inv"),
            Self::And(size) => write!(f, "and{}", size),
            Self::Nand(size) => write!(f, "nand{}", size),
            Self::Or(size) => write!(f, "or{}", size),
            Self::Nor(size) => write!(f, "nor{}", size),
        }
    }
}

impl Design for Stdcell {
    fn name(&self) -> ShrString {
        self.name.clone()
    }

    fn ports(&self) -> &[Shr<Port>] {
        &self.ports
    }
}

impl Primitive for Stdcell {
    fn netlist(&self) -> &Subckt {
        &self.netlist
    }
}