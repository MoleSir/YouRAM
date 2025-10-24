use crate::circuit::{Shr, ShrString};
use super::Net;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortDirection {
    Input,
    Output,
    InOut,
    Vdd, 
    Gnd,
}

pub struct Port {
    pub name: ShrString,
    pub direction: PortDirection,
    pub net: Option<Shr<Net>>,
}

impl Port {
    pub fn new<S: Into<ShrString>>(name: S, direction: PortDirection) -> Shr<Self> {
        Shr::new( Self { name: name.into(), direction, net: None } )
    }

    pub fn is_input(&self) -> bool {
        self.direction == PortDirection::Input
    }

    pub fn is_output(&self) -> bool {
        self.direction == PortDirection::Output
    }

    pub fn is_source(&self) -> bool {
        self.direction == PortDirection::Vdd || self.direction == PortDirection::Gnd
    }

    pub fn is_vdd(&self) -> bool {
        self.direction == PortDirection::Vdd
    }

    pub fn is_gnd(&self) -> bool {
        self.direction == PortDirection::Gnd
    }

    pub fn connected(&self) -> bool {
        self.net.is_some()
    }

    pub fn set_connected_net(&mut self, net: Shr<Net>) {
        self.net = Some(net)
    }
}
