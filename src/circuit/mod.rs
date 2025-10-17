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
    Stdcell(Shr<Stdcell>),
    Leafcell(Shr<Leafcell>),
}

impl Into<ShrCircuit> for Shr<Box<dyn Modular>> {
    fn into(self) -> ShrCircuit {
        ShrCircuit::Module(self)
    }
}

impl Into<ShrCircuit> for Shr<Stdcell> {
    fn into(self) -> ShrCircuit {
        ShrCircuit::Stdcell(self)
    }
}

impl Into<ShrCircuit> for Shr<Leafcell> {
    fn into(self) -> ShrCircuit {
        ShrCircuit::Leafcell(self)
    }
}

impl ShrCircuit {
    pub fn name(&self) -> ShrString {
        match self {
            Self::Module(module) => module.read().name(),
            Self::Stdcell(stdcell) => stdcell.read().name(),
            Self::Leafcell(leafcell) => leafcell.read().name(),
        }
    }
    
    pub fn is_moudle(&self) -> bool {
        match self {
            Self::Module(_) => true,
            _ => false,
        }
    }

    pub fn is_stdcell(&self) -> bool {
        match self {
            Self::Stdcell(_) => true,
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

    pub fn stdcell(&self) -> Option<Shr<Stdcell>> {
        match self {
            Self::Stdcell(stdcell) => Some(stdcell.clone()),
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
