mod buffer;
mod decoder;
use anyhow::Context;
pub use buffer::*;
pub use decoder::*;
use tracing::{debug, warn};

use std::{collections::HashMap, fmt::Debug, ops::Deref};
use super::{Circuit, CircuitError, CircuitFactory, Design, DriveStrength, Instance, ModuleArg, Net, Pin, Port, PortDirection, Shr, ShrString, StdcellKind, StdcellPort};

pub trait Modular: Design {
    fn instances(&self) -> &[Shr<Instance>];
    fn circuits(&self) -> &[Shr<Circuit>];
}

#[derive(Debug)]
pub struct Module<A> {
    pub name: ShrString,
    pub ports: Vec<Shr<Port>>,
    pub instances: Vec<Shr<Instance>>,
    pub circuits: Vec<Shr<Circuit>>,
    pub nets: HashMap<ShrString, Shr<Net>>,
    pub args: A,
}

pub trait AsInstance<A> : Clone {
    fn as_instance(self, module: &Module<A>) -> Result<Shr<Instance>, CircuitError>;
}

pub trait AsPin : Clone {
    fn as_pin(self, instance: &Shr<Instance>) -> Result<Shr<Pin>, CircuitError>;
}

impl<A> Module<A> {
    pub fn new<S: Into<ShrString>>(name: S, args: A) -> Self {
        Self {
            name: name.into(), 
            ports: Vec::new(),
            instances: Vec::new(),
            circuits: Vec::new(),
            nets: HashMap::new(),
            args
        }
    }

    pub fn add_port<S: Into<ShrString>>(&mut self, name: S, direction: PortDirection) -> anyhow::Result<Shr<Port>> {
        let name: ShrString = name.into();
        debug!("add port {} to circuit {}", name, self.name);
        
        (|| {
            if self.ports.iter().any(|port| port.read().name == name) {
                Err(CircuitError::AddDuplicatePort(name.to_string()))
            } else {
                // add a port and net with the same name
                let port = Port::new(name.clone(), direction);
                let net = self.add_net(port.read().name.clone());
                port.wrire().set_connected_net(net.clone());
                net.wrire().add_connection(port.clone());
                self.ports.push(port.clone());
                Ok(port)
            }
        })()
        .with_context(|| format!("add port {} to circuit {}", name, self.name))
    }

    pub fn add_module(&mut self, arg: impl ModuleArg + 'static, factory: &mut CircuitFactory) -> anyhow::Result<Shr<Circuit>> {   
        (|| -> anyhow::Result<Shr<Circuit>> {
            debug!("add sub module to circuit {}", self.name);
            let module = factory.module(arg)?;
            self.add_circuit(module.clone());
            Ok(module) 
        })()  
        .with_context(|| format!("add sub module to circuit {}", self.name))
    }

    pub fn add_stdcell(&mut self, kind: StdcellKind, drive_strength: DriveStrength, factory: &mut CircuitFactory) -> anyhow::Result<Shr<Circuit>> {
        (|| -> anyhow::Result<Shr<Circuit>> {
            debug!("add stdcell {}, {} to circuit {}", kind, drive_strength, self.name);
            let stdcell = factory.stdcell(kind, drive_strength)?;
            self.add_circuit(stdcell.clone());
            Ok(stdcell)
        })()  
        .with_context(|| format!("add stdcell ({},{}) to circuit {}", kind, drive_strength, self.name))
    }

    pub fn add_circuit(&mut self, circuit: Shr<Circuit>) {
        self.circuits.push(circuit);
    }

    pub fn add_instance<S>(&mut self, name: S, circuit: Shr<Circuit>) -> anyhow::Result<Shr<Instance>> 
    where 
        S: Into<ShrString>,
    {
        let name: ShrString = name.into();
        debug!("add instance {} to circuit {}", name, self.name);

        (|| {    
            if self.instances.iter().any(|inst| inst.read().name == name) {
                return Err(CircuitError::AddDuplicateInstance(name.to_string()));
            }
    
            // Create a new instance
            let instance = Instance::new(name.clone(), circuit);
            self.instances.push(instance.clone());
            
            Ok(instance)
        })()
        .with_context(|| format!("add instance {} to circuit {}", name, self.name))
    }

    pub fn add_net<S: Into<ShrString>>(&mut self, name: S) -> Shr<Net> {
        let name_str = name.into();
        if let Some(net) = self.nets.get(&name_str) {
            net.clone()
        } else {
            let net = Net::new(name_str.clone());
            self.nets.insert(name_str, net.clone());
            net
        }
    }

    pub fn connect_instance_in_order<'a, T, S, I>(&mut self, instance: T, nets: I) -> anyhow::Result<()> 
    where 
        T: AsInstance<A>,
        S: Into<ShrString>,
        I: ExactSizeIterator<Item = S>,
    {        
        let instance = instance.as_instance(self)?;
        if instance.read().template_circuit.read().is_stdcell() {
            warn!("connect a stdcell instance in order!!!");
        }

        (|| -> anyhow::Result<()>  {
            if instance.read().pins.len() != nets.len() {
                Err(CircuitError::PinSizeUnmatch(instance.read().pins.len(), nets.len()))?;
            }
            
            debug!("connect instance {} to circuit {}", instance.read().name, self.name);
            for (pin, net) in instance.read().pins.iter().zip(nets) {
                self.connect_pin_with_net(instance.clone(), pin, net)?;
            }
    
            Ok(())
        })()
        .with_context(|| format!("connect instance {} to circuit {}", instance.read().name, self.name))
    }

    pub fn connect_instance<'a, T, P, S, I>(&mut self, instance: T, pin_to_nets: I) -> anyhow::Result<()> 
    where 
        T: AsInstance<A>,
        P: AsPin,
        S: Into<ShrString>,
        I: ExactSizeIterator<Item = (P, S)>,
    {
        let instance = instance.as_instance(self)?;
        (|| -> anyhow::Result<()>  {
            if instance.read().pins.len() != pin_to_nets.len() {
                Err(CircuitError::PinSizeUnmatch(instance.read().pins.len(), pin_to_nets.len()))?;
            }

            debug!("connect instance {} to circuit {}", instance.read().name, self.name);
            for (pin, net) in pin_to_nets {
                self.connect_pin_with_net(instance.clone(), pin, net)?;
            }
            Ok(())
        })()
        .with_context(|| format!("connect instance {} to circuit {}", instance.read().name, self.name))
    }

    pub fn connect_pin_with_net(&mut self, instance: impl AsInstance<A>, pin: impl AsPin, net: impl Into<ShrString>) -> Result<Shr<Net>, CircuitError> {
        let instance = instance.as_instance(self)?;
        
        let pin = pin.as_pin(&instance)?;
        let net = self.add_net(net);
        
        pin.wrire().set_connected_net(net.clone());
        net.wrire().add_connection(pin.clone());

        debug!("connect pin {} with net {}", pin.read().name, net.read().name);
        Ok(net)
    }
}

impl<A> AsInstance<A> for Shr<Instance> {
    fn as_instance(self, _: &Module<A>) -> Result<Self, CircuitError> {
        // Check instance
        Ok(self)
    }
} 

impl<A> AsInstance<A> for &str {
    fn as_instance(self, module: &Module<A>) -> Result<Shr<Instance>, CircuitError> {
        module.instances.iter()
            .find(|instance| instance.read().name == self)
            .cloned()
            .ok_or_else(|| CircuitError::InstanceNotFound(self.to_string(), module.name.to_string()))
    }
}

impl AsPin for &str {
    fn as_pin(self, instance: &Shr<Instance>) -> Result<Shr<Pin>, CircuitError> {
        instance.read()
            .get_pin(self)
            .ok_or_else(|| CircuitError::PinNotFound(self.to_string(), instance.read().name.to_string()))
    }
}

impl AsPin for Shr<Pin> {
    fn as_pin(self, _: &Shr<Instance>) -> Result<Shr<Pin>, CircuitError> {
        // check self
        Ok(self)
    }
}

impl AsPin for &Shr<Pin> {
    fn as_pin(self, _: &Shr<Instance>) -> Result<Shr<Pin>, CircuitError> {
        // check self
        Ok(self.clone())
    }
}

impl AsPin for StdcellPort {
    fn as_pin(self, instance: &Shr<Instance>) -> Result<Shr<Pin>, CircuitError> {
        if let Circuit::Stdcell(stdcell) = instance.read().template_circuit.read().deref() {
            let name = match self {
                StdcellPort::Input(order) => stdcell.input_pn(order)?,
                StdcellPort::Output => stdcell.output_pn(),
                StdcellPort::Vdd => stdcell.vdd_pn(),
                StdcellPort::Gnd => stdcell.gnd_pn(),
            };
            name.as_str().as_pin(instance)
        } else {
            Err(CircuitError::Messgae(format!("expect stdcell")))
        }
    }
}

impl<A: Debug> Design for Module<A> {
    fn name(&self) -> ShrString {
        self.name.clone()
    }

    fn ports(&self) -> &[Shr<Port>] {
        &self.ports
    }
}

impl<A: Debug> Modular for Module<A> {
    fn circuits(&self) -> &[Shr<Circuit>] {
        &self.circuits
    }

    fn instances(&self) -> &[Shr<Instance>] {
        &self.instances
    }
}
