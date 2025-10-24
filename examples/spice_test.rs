#![allow(unused)]
use std::{io::Write, sync::Arc};
use reda_unit::t;
use tracing::Level;
use tracing_subscriber::fmt::format;
use youram::{circuit::{BankArg, BitcellArrayRecursiveArg, BufferArg, CircuitFactory, ControlLogic, ControlLogicArg, CoreArg, DataPathArg, Decoder, DecoderArg, DriveStrength, FanoutBufferArg, SramArg}, export, pdk::{Enviroment, Pdk}, simulate::{CircuitSimulator, NgSpice, VoltageAtMeas}, ErrorContext};

fn main_result() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    let pdk = Arc::new(Pdk::load("./platforms/nangate45").context("load pdk")?);
    let mut factory = CircuitFactory::new(pdk.clone());
    let decoder = factory.module(DecoderArg::new(3))?;

    let pvt = pdk.pvt();
    let env = Enviroment::new(pvt.clone(), t!(0.5 n), 0.0.into());

    let mut simulator = CircuitSimulator::create(
        decoder.clone(), 
        env, 
        pdk.clone(), 
        "./temp/simulate.sp", 
        "./temp/decoder.sp"
    )?;

    let decoder_ref = decoder.read();
    simulator.write_logic1_stimulate(Decoder::address_pn(0));
    simulator.write_logic1_stimulate(Decoder::address_pn(1));
    simulator.write_logic1_stimulate(Decoder::address_pn(2));

    for i in 0..8 {
        let meas = VoltageAtMeas::new(format!("output{i}"), Decoder::output_pn(i).to_string(), t!(10. n));
        simulator.write_measurement(Box::new(meas))?;
    }
    simulator.write_trans(t!(0.5 n), t!(0.0), t!(15. n))?;


    let result = simulator.simulate(NgSpice, "./temp")?;

    for (name, value) in result {
        println!("{name}: {value}");
    }

    std::io::stdout().flush()?;
    Ok(())
}

fn main() {
    if let Err(e) = main_result() {
        eprint!("Err: {}\n", e);
    }
}