mod srdstring;
mod shared;
mod module;
mod primitive;
mod base;
mod error;
mod factory;

use std::fmt::Debug;

pub use shared::*;
pub use srdstring::ShrString;
pub use module::*;
pub use primitive::*;
pub use base::*;
pub use error::*;
pub use factory::*;

pub trait Design: Debug {
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

#[derive(Debug)]
pub enum Circuit {
    Module(Box<dyn Modular>),
    Stdcell(Stdcell),
    Leafcell(Leafcell),
}

impl Design for Circuit {
    fn name(&self) -> ShrString {
        match self {
            Circuit::Module(module) => module.name(),
            Circuit::Stdcell(stdcell) => stdcell.name(),
            Circuit::Leafcell(leafcell) => leafcell.name(),
        }
    }

    fn ports(&self) -> &[Shr<Port>] {
        match self {
            Circuit::Module(module) => module.ports(),
            Circuit::Stdcell(stdcell) => stdcell.ports(),
            Circuit::Leafcell(leafcell) => leafcell.ports(),
        }
    }
}

impl Circuit {
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