use std::{path::{Path, PathBuf}, sync::Arc};
use reda_unit::{t, Time};
use serde::{Deserialize, Serialize};
use tracing::{info, Level};
use clap::Parser;
use youram::{
    charz::{FunctionCharz, FunctionCharzPolicy, RandomPolicy}, 
    circuit::{CircuitFactory, SramArg}, 
    export, 
    pdk::{Enviroment, Pdk}, 
    simulate::{SpiceCommand, NgSpice}, 
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
        let policy = parse_function_test_policy(&function_test)?;
        let period = config.period;
        let command = config.spice_command()?;

        // load simulate config in pdk
        let pvt = pdk.pvt();
        let output_load = pdk.default_fanout_load().unwrap_or(0.0.into());
        let input_slew = period / 20.0;
        let env = Enviroment::new(pvt.clone(), input_slew, output_load);

        // run functional test
        FunctionCharz::config()
            .sram(sram.clone())
            .period(period)
            .env(env)
            .pdk(pdk.clone())
            .policy_box(policy)
            .command_box(command)
            .temp_folder(config.temp_folder_path())
            .test()?;
    }

    // write
    if config.export_spice {
        let spice_file = config.join_output(format!("{}.sp", sram.read().name));
        export::write_spice(sram.clone(), spice_file)?;
    }
    
    if config.export_verilog {
        let verilog_file = config.join_output(format!("{}.v", sram.read().name));
        export::write_verilog(sram.clone(), verilog_file)?;
    }

    if config.export_liberty {
        let liberty_file = config.join_output(format!("{}.lib", sram.read().name));
        let command = config.spice_command()?;
        export::write_liberty(
            sram.clone(), 
            liberty_file, 
            config.period, 
            pdk.clone(), 
            command, 
            config.temp_folder_path()
        )?;
    }

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
    
    #[serde(default = "default_spice_command")]
    pub spice_command: String,

    #[serde(default = "default_period")]
    pub period: Time,
    
    pub function_test: Option<String>,

    #[serde(default = "const_true")]
    pub export_spice: bool,

    #[serde(default = "const_true")]
    pub export_verilog: bool,

    #[serde(default = "const_false")]
    pub export_liberty: bool,
}

fn parse_function_test_policy(policy: &str) -> Result<Box<dyn FunctionCharzPolicy>, Box<dyn std::error::Error>> {
    match policy {
        "random" => Ok(Box::new(RandomPolicy)),
        _ => Err(format!("Un support function test policy: {}", policy))?,
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

    pub fn join_output(&self, path: impl AsRef<Path>) -> PathBuf {
        self.output_path.join(path.as_ref())
    }

    pub fn spice_command(&self) -> Result<Box<dyn SpiceCommand>, Box<dyn std::error::Error>> {
        match self.spice_command.as_str() {
            "ngspice" => Ok(Box::new(NgSpice)),
            _ => Err(format!("Un support spice executor: {}", self.spice_command.as_str()))?,
        }
    }
}

fn default_spice_command() -> String {
    "ngspice".to_string()
}

fn default_period() -> Time {
    t!(10 n)
}

const fn const_true() -> bool {
    true
}

const fn const_false() -> bool {
    false
}