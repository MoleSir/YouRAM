use std::sync::Arc;
use reda_unit::t;
use tracing::Level;
use youram::{
    charz::function::{FunctionTestBuilder, RandomPolicy}, 
    circuit::{CircuitFactory, SramArg}, 
    pdk::{Enviroment, Pdk}, 
    simulate::NgSpice, ErrorContext
};

const PDK: &str = "./platforms/nangate45";
const TEMP: &str = "./temp";
const ADDRESS_WIDTH: usize = 4;
const WORD_WIDTH: usize = 4;

fn main_result() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    let pdk = Arc::new(Pdk::load(PDK).context("load pdk")?);
    let mut factory = CircuitFactory::new(pdk.clone());
    let sram = factory.module(SramArg::new(ADDRESS_WIDTH, WORD_WIDTH))?;
    let pvt = pdk.pvt();
    let env = Enviroment::new(pvt.clone(), t!(0.5 n), 0.0.into());

    let config = FunctionTestBuilder::default()
        .sram(sram.clone())
        .period(t!(10. n))
        .env(env)
        .pdk(pdk)
        .policy(RandomPolicy)
        .command(NgSpice)
        .temp_folder(TEMP)
        .build()?;

    let res = config.test()?;

    assert!(res);

    Ok(())
}

#[test]
fn main() {
    if let Err(e) = main_result() {
        eprint!("Err: {}\n", e);
    }
}