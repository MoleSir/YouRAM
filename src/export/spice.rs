use std::collections::HashSet;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::ops::Deref;
use std::path::Path;
use anyhow::Context;
use reda_sp::ToSpice;
use tracing::{debug, info};
use crate::circuit::{Circuit, CircuitError, Design, Primitive, Shr, ShrString};

pub fn write_spice<P: AsRef<Path>>(circuit: Shr<Circuit>, path: P) -> anyhow::Result<()> {
    let path = path.as_ref();
    info!("write circuit {} to {:?}", circuit.read().name(), path);
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    let mut exported = HashSet::new();
    write_spice_recursive(&mut writer, &circuit, &mut exported).context(format!("export {}", circuit.read().name()))?;
    Ok(())
}

fn write_spice_recursive<W: Write>(
    writer: &mut W,
    circuit: &Shr<Circuit>,
    exported: &mut HashSet<ShrString>,
) -> anyhow::Result<()> {
    let name = circuit.read().name();
    if !exported.insert(name.clone()) {
        return Ok(()); 
    }

    match circuit.read().deref() {
        Circuit::Module(module) => {
            debug!("write module {}", name);
            for sub in module.circuits() {
                write_spice_recursive(writer, sub, exported).context(format!("export {}", circuit.read().name()))?;
            }

            // .SUBCKT header
            let ports = module.ports();
            let port_names: Vec<_> = ports.iter().map(|p| p.read().name.to_string()).collect();
            writeln!(writer, ".SUBCKT {} {}", module.name(), port_names.join(" "))?;

            // Instance
            for inst in module.instances() {            
                let inst = inst.read();
                
                let mut pin_nets = Vec::new();
                for pin in inst.pins.iter() {
                    match &pin.read().net {
                        Some(net) => pin_nets.push(net.read().name.to_string()),
                        None => return Err(CircuitError::InstanceNotConnected(inst.name.to_string()))?,
                    }
                }

                let subckt_name = inst.template_circuit.read().name();
                writeln!(writer, "X{} {} {}", inst.name, pin_nets.join(" "), subckt_name)?;
            }

            writeln!(writer, ".ENDS {}\n", module.name())?;
        }
        Circuit::Stdcell(stdcell) => {
            debug!("write stdcell {}", name);
            let spice = stdcell.netlist.to_spice();
            writer.write_all(spice.as_bytes())?;
            writer.write_all("\n\n".as_bytes())?;
        }
        Circuit::Leafcell(leafcell) => {
            debug!("write leafcell {}", name);
            let spice = leafcell.netlist().to_spice();
            writer.write_all(spice.as_bytes())?;
            writer.write_all("\n\n".as_bytes())?;
        }
    }

    Ok(())
}
