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

pub enum Circuit {
    Module(Box<dyn Modular>),
    Stdcell(Stdcell),
    Leafcell(Leafcell),
}

impl Design for Circuit {
    fn ports(&self) -> &[Shr<Port>] {
        match self {
            Circuit::Module(module) => module.ports(),
            Circuit::Stdcell(stdcell) => stdcell.ports(),
            Circuit::Leafcell(leafcell) => leafcell.ports(),
        }
    }
}

impl Circuit {
    pub fn moudle(&self) -> Option<&Box<dyn Modular>> {
        match self {
            Self::Module(module) => Some(module),
            _ => None,
        }
    }

    pub fn stdcell(&self) -> Option<&Stdcell> {
        match self {
            Self::Stdcell(stdcell) => Some(stdcell),
            _ => None,
        }
    }

    pub fn leafcell(&self) -> Option<&Leafcell> {
        match self {
            Self::Leafcell(leafcell) => Some(leafcell),
            _ => None,
        }
    }
}