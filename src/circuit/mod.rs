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

#[derive(PartialEq, Eq, Hash)]
pub enum ShrCircuit {
    Module(Shr<dyn Modular>),
    Primitive(Shr<dyn Primitive>),
}

impl Into<ShrCircuit> for Shr<dyn Modular> {
    fn into(self) -> ShrCircuit {
        ShrCircuit::Module(self)
    }
}

impl Into<ShrCircuit> for Shr<LogicGate> {
    fn into(self) -> ShrCircuit {
        ShrCircuit::Primitive(self.into())
    }
}

impl Into<ShrCircuit> for Shr<Leafcell> {
    fn into(self) -> ShrCircuit {
        ShrCircuit::Primitive(self.into())
    }
}

impl Into<ShrCircuit> for Shr<Dff> {
    fn into(self) -> ShrCircuit {
        ShrCircuit::Primitive(self.into())
    }
}

impl<A: ModuleArg + 'static> Into<ShrCircuit> for Shr<Module<A>> {
    fn into(self) -> ShrCircuit {
        let module: Shr<dyn Modular> = self.into();
        ShrCircuit::Module(module)
    }
}

impl ShrCircuit {
    pub fn name(&self) -> ShrString {
        match self {
            Self::Module(module) => module.read().name(),
            Self::Primitive(p) => p.read().name(),
        }
    }
    
    pub fn is_moudle(&self) -> bool {
        match self {
            Self::Module(_) => true,
            _ => false,
        }
    }

    pub fn is_primitive(&self) -> bool {
        match self {
            Self::Primitive(_) => true,
            _ => false,
        }
    }

    pub fn moudle(&self) -> Option<Shr<dyn Modular>> {
        match self {
            Self::Module(module) => Some(module.clone()),
            _ => None,
        }
    }

    pub fn primitive(&self) -> Option<Shr<dyn Primitive>> {
        match self {
            Self::Primitive(p) => Some(p.clone()),
            _ => None,
        }
    }
}
