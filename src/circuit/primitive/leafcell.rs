use reda_sp::Subckt;
use crate::circuit::{Design, Port, Shr};

use super::Primitive;

pub const BITCELL_NAME: &'static str = "bitcell";

pub enum Leafcell {
    Bitcell(Bitcell),
}

pub struct Bitcell {
    pub bl: Shr<Port>,
    pub br: Shr<Port>,
    pub wl: Shr<Port>,
    pub vdd: Shr<Port>,
    pub gnd: Shr<Port>,

    pub ports: Vec<Shr<Port>>,
    pub netlist: Subckt,
}

impl Bitcell {
    pub fn new(bl: Shr<Port>, br: Shr<Port>, wl: Shr<Port>, vdd: Shr<Port>, gnd: Shr<Port>, netlist: Subckt) -> Self {
        let ports = vec![
            bl.clone(), br.clone(), wl.clone(), vdd.clone(), gnd.clone(), 
        ];
        Self {
            bl, br, wl, vdd, gnd, ports, netlist
        }
    }
}

impl From<Bitcell> for Leafcell {
    fn from(value: Bitcell) -> Self {
        Self::Bitcell(value)
    }
}

impl Design for Leafcell {
    fn ports(&self) -> &[Shr<Port>] {
        match self {
            Leafcell::Bitcell(bitcell) => &bitcell.ports
        }
    }
}

impl Primitive for Leafcell {
    fn netlist(&self) -> &Subckt {
        match self {
            Leafcell::Bitcell(bitcell) => &bitcell.netlist
        }
    }
}