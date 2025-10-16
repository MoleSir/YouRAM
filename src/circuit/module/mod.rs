use std::collections::HashMap;
use super::{Circuit, CircuitError, Design, Instance, Net, Pin, Port, PortDirection, Shr, ShrString};

pub trait Modular: Design {
    fn name(&self) -> ShrString;
    fn instances(&self) -> &[Shr<Instance>];
    fn circuits(&self) -> &[Shr<Circuit>];
}

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

    pub fn add_port<S: Into<ShrString>>(&mut self, name: S, direction: PortDirection) -> Result<Shr<Port>, CircuitError> {
        let name: ShrString = name.into();
        if self.ports.iter().any(|port| port.read().name == name) {
            Err(CircuitError::AddDuplicatePort(name.to_string()))
        } else {
            // add a port and net with the same name
            let port = Port::new(name, direction);
            let net = self.add_net(port.read().name.clone());
            port.wrire().set_connected_net(net.clone());
            net.wrire().add_connection(port.clone());
            self.ports.push(port.clone());
            Ok(port)
        }
    }

    pub fn add_instance<S, I>(&mut self, name: S, circuit: Shr<Circuit>) -> Result<Shr<Instance>, CircuitError> 
    where 
        S: Into<ShrString>,
        I: Iterator<Item = ShrString>
    {
        let name: ShrString = name.into();
        if self.instances.iter().any(|inst| inst.read().name == name) {
            return Err(CircuitError::AddDuplicateInstance(name.to_string()));
        }

        // Create a new instance
        let instance = Instance::new(name, circuit);
        self.instances.push(instance.clone());

        Ok(instance)
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

    pub fn connect_instance_in_order<'a>(&mut self, instance: impl AsInstance<A>, nets: impl ExactSizeIterator<Item = impl Into<ShrString>>) -> Result<(), CircuitError> {
        let instance = instance.as_instance(self)?;
        if instance.read().pins.len() != nets.len() {
            Err(CircuitError::PinSizeUnmatch(instance.read().pins.len(), nets.len()))?;
        }
        
        for (pin, net) in instance.read().pins.iter().zip(nets) {
            self.connect_pin_with_net(instance.clone(), pin, net)?;
        }

        Ok(())
    }

    pub fn connect_instance<'a>(&mut self, instance: impl AsInstance<A>, pin_to_nets: impl Iterator<Item = (&'a str, impl Into<ShrString>)>) -> Result<(), CircuitError> {
        for (pin, net) in pin_to_nets {
            self.connect_pin_with_net(instance.clone(), pin, net)?;
        }
        Ok(())
    }

    pub fn connect_pin_with_net(&mut self, instance: impl AsInstance<A>, pin: impl AsPin, net: impl Into<ShrString>) -> Result<Shr<Net>, CircuitError> {
        // TODO: check instance 
        let instance = instance.as_instance(self)?;
        let pin = pin.as_pin(&instance)?;
        
        let net = self.add_net(net);
        pin.wrire().set_connected_net(net.clone());
        net.wrire().add_connection(pin);

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

impl<A> Design for Module<A> {
    fn ports(&self) -> &[Shr<Port>] {
        &self.ports
    }
}

impl<A> Modular for Module<A> {
    fn name(&self) -> ShrString {
        self.name.clone()
    }

    fn circuits(&self) -> &[Shr<Circuit>] {
        &self.circuits
    }

    fn instances(&self) -> &[Shr<Instance>] {
        &self.instances
    }
}
