#![allow(unused)]
use std::sync::Arc;
use tracing::Level;
use youram::{circuit::{BitcellArrayRecursiveArg, BufferArg, CircuitFactory, ControlLogic, ControlLogicArg, DecoderArg, DriveStrength}, export, pdk::Pdk, ErrorContext};

fn main_result() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_target(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    let pdk = Arc::new(Pdk::load("./platforms/nangate45").context("load pdk")?);
    let mut factory = CircuitFactory::new(pdk);

    // let logic = factory.module(ControlLogicArg::new())?;
    // export::write_spice(logic, "./temp/logic.sp").context("export spice")?;

    // let decoder = factory.module(DecoderArg::new(10))?;
    // export::write_spice(decoder, "./temp/decoder.sp").context("export spice")?;

    let array = factory.module(BitcellArrayRecursiveArg::new(256, 256))?;
    export::write_spice(array, "./temp/bitcellarray.sp").context("export spice")?;

    Ok(())
}

fn main() {
    if let Err(e) = main_result() {
        eprint!("Err: {:?}\n", e);
    }
}