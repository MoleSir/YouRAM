use reda_sp::Subckt;
use crate::circuit::{Design, Port, Shr};

use super::Primitive;

pub const BITCELL_NAME: &'static str = "bitcell";
pub const SENSE_AMP_NAME: &'static str = "sense_amp";
pub const WRITE_DRIVER_NAME: &'static str = "write_driver";
pub const COLUMN_TRI_GATE_NAME: &'static str = "column_tri_gate";

#[derive(Debug)]
pub enum Leafcell {
    Bitcell(Bitcell),
    SenseAmp(SenseAmp),
    WriteDriver(WriteDriver),
    ColumnTriGate(ColumnTriGate),
}

#[derive(Debug)]
pub struct Bitcell {
    pub bitline: Shr<Port>,
    pub bitline_bar: Shr<Port>,
    pub word_line: Shr<Port>,
    pub vdd: Shr<Port>,
    pub gnd: Shr<Port>,

    pub ports: Vec<Shr<Port>>,
    pub netlist: Subckt,
}

#[derive(Debug)]
pub struct SenseAmp {
    pub bitline: Shr<Port>,
    pub bitline_bar: Shr<Port>,
    pub data_output: Shr<Port>,
    pub enable: Shr<Port>,
    pub vdd: Shr<Port>,
    pub gnd: Shr<Port>,

    pub ports: Vec<Shr<Port>>,
    pub netlist: Subckt,
}

#[derive(Debug)]
pub struct WriteDriver {
    pub data_input: Shr<Port>,
    pub bitline: Shr<Port>,
    pub bitline_bar: Shr<Port>,
    pub enable: Shr<Port>,
    pub vdd: Shr<Port>,
    pub gnd: Shr<Port>,

    pub ports: Vec<Shr<Port>>,
    pub netlist: Subckt,
}

#[derive(Debug)]
pub struct ColumnTriGate {
    pub bitline: Shr<Port>,
    pub bitline_bar: Shr<Port>,
    pub bitline_output: Shr<Port>,
    pub bitline_bar_output: Shr<Port>,
    pub select: Shr<Port>,
    pub vdd: Shr<Port>,
    pub gnd: Shr<Port>,

    pub ports: Vec<Shr<Port>>,
    pub netlist: Subckt,
}

impl Bitcell {
    pub fn new(bitline: Shr<Port>, bitline_bar: Shr<Port>, word_line: Shr<Port>, vdd: Shr<Port>, gnd: Shr<Port>, netlist: Subckt) -> Self {
        let ports = vec![
            bitline.clone(), bitline_bar.clone(), word_line.clone(), vdd.clone(), gnd.clone(), 
        ];
        Self {
            bitline, bitline_bar, word_line, vdd, gnd, ports, netlist
        }
    }
}

impl SenseAmp {
    pub fn new(
        bitline: Shr<Port>,
        bitline_bar: Shr<Port>,
        data_output: Shr<Port>,
        enable: Shr<Port>,
        vdd: Shr<Port>,
        gnd: Shr<Port>,
        netlist: Subckt,
    ) -> Self {
        let ports = vec![
            bitline.clone(),
            bitline_bar.clone(),
            enable.clone(),
            vdd.clone(),
            gnd.clone(),
        ];
        Self {
            bitline,
            bitline_bar,
            data_output,
            enable,
            vdd,
            gnd,
            ports,
            netlist,
        }
    }
}

impl WriteDriver {
    pub fn new(
        data_input: Shr<Port>,
        bitline: Shr<Port>,
        bitline_bar: Shr<Port>,
        enable: Shr<Port>,
        vdd: Shr<Port>,
        gnd: Shr<Port>,
        netlist: Subckt,
    ) -> Self {
        let ports = vec![
            data_input.clone(),
            bitline.clone(),
            bitline_bar.clone(),
            enable.clone(),
            vdd.clone(),
            gnd.clone(),
        ];
        Self {
            data_input,
            bitline,
            bitline_bar,
            enable,
            vdd,
            gnd,
            ports,
            netlist,
        }
    }
}

impl ColumnTriGate {
    pub fn new(
        bitline: Shr<Port>,
        bitline_bar: Shr<Port>,
        bitline_output: Shr<Port>,
        bitline_bar_output: Shr<Port>,
        select: Shr<Port>,
        vdd: Shr<Port>,
        gnd: Shr<Port>,
        netlist: Subckt,
    ) -> Self {
        let ports = vec![
            bitline.clone(),
            bitline_bar.clone(),
            bitline_output.clone(),
            bitline_bar_output.clone(),
            select.clone(),
            vdd.clone(),
            gnd.clone(),
        ];
        Self {
            bitline,
            bitline_bar,
            bitline_output,
            bitline_bar_output,
            select,
            vdd,
            gnd,
            ports,
            netlist,
        }
    }
}

impl From<Bitcell> for Leafcell {
    fn from(value: Bitcell) -> Self {
        Self::Bitcell(value)
    }
}

impl From<WriteDriver> for Leafcell {
    fn from(value: WriteDriver) -> Self {
        Self::WriteDriver(value)
    }
}

impl From<SenseAmp> for Leafcell {
    fn from(value: SenseAmp) -> Self {
        Self::SenseAmp(value)
    }
}

impl From<ColumnTriGate> for Leafcell {
    fn from(value: ColumnTriGate) -> Self {
        Self::ColumnTriGate(value)
    }
}

impl Design for Leafcell {
    fn name(&self) -> crate::circuit::ShrString {
        match self {
            Self::Bitcell(_) => BITCELL_NAME.into(),
            Self::SenseAmp(_) => SENSE_AMP_NAME.into(),
            Self::WriteDriver(_) => WRITE_DRIVER_NAME.into(),
            Self::ColumnTriGate(_) => COLUMN_TRI_GATE_NAME.into(),
        }
    }

    fn ports(&self) -> &[Shr<Port>] {
        match self {
            Self::Bitcell(l) => &l.ports,
            Self::SenseAmp(l) => &l.ports,
            Self::WriteDriver(l) => &l.ports,
            Self::ColumnTriGate(l) => &l.ports,
        }
    }
}

impl Primitive for Leafcell {
    fn netlist(&self) -> &Subckt {
        match self {
            Self::Bitcell(l) => &l.netlist,
            Self::SenseAmp(l) => &l.netlist,
            Self::WriteDriver(l) => &l.netlist,
            Self::ColumnTriGate(l) => &l.netlist,
        }
    }
}