use reda_sp::Subckt;
use crate::circuit::{Design, Port, Shr};

use super::Primitive;

pub enum Leafcell {
    Bitcell(Bitcell),
    SenseAmp(SenseAmp),
    WriteDriver(WriteDriver),
    ColumnTriGate(ColumnTriGate),
    Precharge(Precharge),
}

macro_rules! define_leafcell {
    ($name:ident, $($port:ident),+ $(,)?) => {
        pub struct $name {
            $(pub $port: Shr<Port>,)+
            pub ports: ::std::vec::Vec<Shr<Port>>,
            pub netlist: Subckt,
        }

        impl $name {
            pub fn new($($port: Shr<Port>,)+ netlist: Subckt) -> Self {
                let ports = ::std::vec![ $($port.clone(),)+ ];
                Self { $($port,)+ ports, netlist }
            }
        }

        impl From<$name> for Leafcell {
            fn from(value: $name) -> Self {
                Self::$name(value)
            }
        }
    };
}

pub const BITCELL_NAME: &'static str = "bitcell";
pub const SENSE_AMP_NAME: &'static str = "sense_amp";
pub const WRITE_DRIVER_NAME: &'static str = "write_driver";
pub const COLUMN_TRI_GATE_NAME: &'static str = "column_trigate";
pub const PRECHARGE_NAME: &'static str = "precharge";

define_leafcell!(Bitcell, bitline, bitline_bar, word_line, vdd, gnd);
define_leafcell!(SenseAmp, bitline, bitline_bar, data_output, enable, vdd, gnd);
define_leafcell!(WriteDriver, data_input, bitline, bitline_bar, enable, vdd, gnd);
define_leafcell!(ColumnTriGate, bitline, bitline_bar, bitline_output, bitline_bar_output, select, vdd, gnd);
define_leafcell!(Precharge, bitline, bitline_bar, enable, vdd);

impl Design for Leafcell {
    fn name(&self) -> crate::circuit::ShrString {
        match self {
            Self::Bitcell(_) => BITCELL_NAME.into(),
            Self::SenseAmp(_) => SENSE_AMP_NAME.into(),
            Self::WriteDriver(_) => WRITE_DRIVER_NAME.into(),
            Self::ColumnTriGate(_) => COLUMN_TRI_GATE_NAME.into(),
            Self::Precharge(_) => PRECHARGE_NAME.into(),
        }
    }

    fn ports(&self) -> &[Shr<Port>] {
        match self {
            Self::Bitcell(l) => &l.ports,
            Self::SenseAmp(l) => &l.ports,
            Self::WriteDriver(l) => &l.ports,
            Self::ColumnTriGate(l) => &l.ports,
            Self::Precharge(l) => &l.ports,
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
            Self::Precharge(l) => &l.netlist,
        }
    }
}