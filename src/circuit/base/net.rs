use crate::circuit::{ShrString, Shr};
use super::{Pin, Port};

#[derive(Debug)]
pub struct Net {
    pub name: ShrString,
    pub connections: Vec<NetNode>
}

#[derive(Debug)]
pub enum NetNode {
    Port(Shr<Port>),
    Pin(Shr<Pin>),
}

impl Into<NetNode> for Shr<Port> {
    fn into(self) -> NetNode {
        NetNode::Port(self)
    }
}

impl Into<NetNode> for Shr<Pin> {
    fn into(self) -> NetNode {
        NetNode::Pin(self)
    }
}

impl Net {
    pub fn new<S: Into<ShrString>>(name: S) -> Shr<Self> {
        Shr::new( Self { name: name.into(), connections: vec![] } )
    }

    pub fn add_connection<N: Into<NetNode>>(&mut self, node: N) {
        self.connections.push(node.into());
    }

    pub fn connect(&mut self, node1: impl Into<NetNode>, node2: impl Into<NetNode>) {
        self.add_connection(node1);
        self.add_connection(node2);
    }
}
