#![allow(unused)]
use std::{io::Write, sync::Arc};
use tracing::Level;
use youram::{circuit::{BankArg, BitcellArrayRecursiveArg, BufferArg, CircuitFactory, ControlLogic, ControlLogicArg, CoreArg, DataPathArg, DecoderArg, DriveStrength, FanoutBufferArg, SramArg}, export, pdk::Pdk, ErrorContext};

fn main_result() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
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

    // let array = factory.module(BitcellArrayRecursiveArg::new(256, 256))?;
    // export::write_spice(array, "./temp/bitcellarray.sp").context("export spice")?;

    // let fanout_buf = factory.module(FanoutBufferArg::new(256))?;
    // export::write_spice(fanout_buf, "./temp/fanout_buf.sp").context("export spice")?;

    // let datapath = factory.module(DataPathArg::build(8, 4))?;
    // export::write_spice(datapath, "./temp/datapath.sp").context("export spice")?;

    // let bank = factory.module(BankArg::new(16, 4, 4))?;
    // export::write_spice(bank, "./temp/bank.sp").context("export spice")?;

    // let core = factory.module(CoreArg::new(16, 4, 4))?;
    // export::write_spice(core, "./temp/core.sp").context("export spice")?;

    let sram = factory.module(SramArg::new(10, 8))?;
    export::write_spice(sram, "./temp/sram.sp").context("export spice")?;

    std::io::stdout().flush()?;
    Ok(())
}

fn main() {
    if let Err(e) = main_result() {
        eprint!("Err: {}\n", e);
    }
}