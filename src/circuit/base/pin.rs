use crate::circuit::{ShrString, Shr};

use super::{Net, Port};

pub struct Pin {
    pub name: ShrString,
    pub net: Option<Shr<Net>>,
    pub template_port: Shr<Port>,
}

pub type RcPin = Shr<Pin>;

impl Pin {
    pub fn new<S: Into<ShrString>>(name: S, template_port: Shr<Port>) -> Shr<Self> {
        Shr::new(Self {
            name: name.into(), net: None, template_port
        })
    }

    pub fn connected(&self) -> bool {
        self.net.is_some()
    }

    pub fn set_connected_net(&mut self, net: Shr<Net>) {
        self.net = Some(net)
    }
}
