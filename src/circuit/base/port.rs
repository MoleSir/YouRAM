use crate::circuit::{Shr, ShrString};
use super::Net;

#[derive(Debug, Clone, Copy)]
pub enum PortDirection {
    Input,
    Output,
    InOut,
    Source,
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

    pub fn connected(&self) -> bool {
        self.net.is_some()
    }

    pub fn set_connected_net(&mut self, net: Shr<Net>) {
        self.net = Some(net)
    }
}
