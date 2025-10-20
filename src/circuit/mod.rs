mod srdstring;
mod shared;
mod module;
mod primitive;
mod base;
mod error;
mod factory;

pub use shared::*;
pub use srdstring::ShrString;
pub use module::*;
pub use primitive::*;
pub use base::*;
pub use error::*;
pub use factory::*;

pub trait Design {
    fn name(&self) -> ShrString;
    fn ports(&self) -> &[Shr<Port>];
    fn get_port(&self, name: &str) -> Option<Shr<Port>> {
        for port in self.ports() {
            if port.read().name == name {
                return Some(port.clone());
            }
        }
        None
    }  
}

pub enum ShrCircuit {
    Module(Shr<Box<dyn Modular>>),
    LogicGate(Shr<LogicGate>),
    Dff(Shr<Dff>),
    Leafcell(Shr<Leafcell>),
}

impl Into<ShrCircuit> for Shr<Box<dyn Modular>> {
    fn into(self) -> ShrCircuit {
        ShrCircuit::Module(self)
    }
}

impl Into<ShrCircuit> for Shr<LogicGate> {
    fn into(self) -> ShrCircuit {
        ShrCircuit::LogicGate(self)
    }
}

impl Into<ShrCircuit> for Shr<Leafcell> {
    fn into(self) -> ShrCircuit {
        ShrCircuit::Leafcell(self)
    }
}

impl Into<ShrCircuit> for Shr<Dff> {
    fn into(self) -> ShrCircuit {
        ShrCircuit::Dff(self)
    }
}

impl ShrCircuit {
    pub fn name(&self) -> ShrString {
        match self {
            Self::Module(module) => module.read().name(),
            Self::LogicGate(logicgate) => logicgate.read().name(),
            Self::Dff(dff) => dff.read().name(),
            Self::Leafcell(leafcell) => leafcell.read().name(),
        }
    }
    
    pub fn is_moudle(&self) -> bool {
        match self {
            Self::Module(_) => true,
            _ => false,
        }
    }

    pub fn is_logicgate(&self) -> bool {
        match self {
            Self::LogicGate(_) => true,
            _ => false,
        }
    }

    pub fn is_leafcell(&self) -> bool {
        match self {
            Self::Leafcell(_) => true,
            _ => false,
        }
    }

    pub fn moudle(&self) -> Option<ShrModule> {
        match self {
            Self::Module(module) => Some(module.clone()),
            _ => None,
        }
    }

    pub fn logicgate(&self) -> Option<Shr<LogicGate>> {
        match self {
            Self::LogicGate(logicgate) => Some(logicgate.clone()),
            _ => None,
        }
    }

    pub fn leafcell(&self) -> Option<Shr<Leafcell>> {
        match self {
            Self::Leafcell(leafcell) => Some(leafcell.clone()),
            _ => None,
        }
    }
}
