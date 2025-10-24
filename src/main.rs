use std::{path::PathBuf, sync::Arc};
use reda_unit::Time;
use serde::{Deserialize, Serialize};
use tracing::{info, Level};
use clap::Parser;
use youram::{
    charz::function::{FunctionTestBuilder, FunctionTestPolicy, RandomPolicy}, 
    circuit::{CircuitFactory, SramArg}, 
    export, 
    pdk::{Enviroment, Pdk}, 
    simulate::{ExecuteCommand, NgSpice}, 
    ErrorContext
};

fn main_result() -> Result<(), Box<dyn std::error::Error>> {   
    let args = Args::parse();
    tracing_subscriber::fmt()
        .with_max_level(args.level())
        .with_target(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    // load config
    let config: Config = {
        let context = std::fs::read_to_string(&args.config).context("read config file")?;
        serde_json::from_str(&context).context("parse config file")?
    };
    config.create_output_path()?;

    // load pdk
    let pdk = Arc::new(Pdk::load(&config.pdk_path).context("load pdk")?);
    
    // create sram
    let mut factory = CircuitFactory::new(pdk.clone());
    let sram = factory.module(SramArg::new(config.address_width, config.word_width)).context("create sram")?;

    // test sram
    if let Some(function_test) = &config.function_test {
        let policy = function_test.policy()?;
        let command = function_test.command()?;
        let period = function_test.period;

        // load simulate config in pdk
        let pvt = pdk.pvt();
        let output_load = pdk.default_fanout_load().unwrap_or(0.0.into());
        let input_slew = period / 20.0;
        let env = Enviroment::new(pvt.clone(), input_slew, output_load);

        // run functional test
        let config = FunctionTestBuilder::default()
            .sram(sram.clone())
            .period(period)
            .env(env)
            .pdk(pdk)
            .policy_box(policy)
            .command_box(command)
            .temp_folder(config.temp_folder_path())
            .build()?;

        config.test()?;
    }

    // write spice
    let outptu_file = config.output_path.join(format!("{}.sp", sram.read().name));
    export::write_spice(sram, outptu_file)?;
    
    Ok(())
}

fn main() {
    if let Err(e) = main_result() {
        eprint!("Err: {}\n", e);
    }
}

/// A simple example CLI
#[derive(Parser, Debug)]
#[command(name = "youram")]
#[command(about = "A Sram Compiler", long_about = None)]
struct Args {
    /// Path to the configuration file
    #[arg(short, long)]
    config: String,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

impl Args {
    pub fn level(&self) -> Level {
        if self.verbose { Level::DEBUG } else { Level::INFO }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub pdk_path: PathBuf,
    pub output_path: PathBuf,
    pub address_width: usize,
    pub word_width: usize,
    pub function_test: Option<FunctionTestConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FunctionTestConfig {
    period: Time,
    policy: String,
    command: String,
}

impl FunctionTestConfig {
    pub fn policy(&self) -> Result<Box<dyn FunctionTestPolicy>, Box<dyn std::error::Error>> {
        match self.policy.as_str() {
            "random" => Ok(Box::new(RandomPolicy)),
            _ => Err(format!("Un support function test policy: {}", self.policy.as_str()))?,
        }
    }

    pub fn command(&self) -> Result<Box<dyn ExecuteCommand>, Box<dyn std::error::Error>> {
        match self.command.as_str() {
            "ngspice" => Ok(Box::new(NgSpice)),
            _ => Err(format!("Un support spice executor: {}", self.command.as_str()))?,
        }
    }
}

impl Config {
    pub fn create_output_path(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Ensure output directory exists
        if !self.output_path.exists() {
            std::fs::create_dir_all(&self.output_path)?;
            info!("created output directory: {:?}", self.output_path);
        }

        // Ensure temp directory exists
        let temp_path = self.temp_folder_path();
        if !temp_path.exists() {
            std::fs::create_dir_all(&temp_path)?;
            info!("created temp directory: {:?}", temp_path);
        }

        Ok(())
    }

    pub fn temp_folder_path(&self) -> PathBuf {
        self.output_path.join("temp")
    }
}
