use std::{collections::HashMap, path::Path, sync::Arc};
use rand::Rng;
use reda_unit::{t, v, Number, Time, Voltage};
use tracing::warn;
use crate::{circuit::{Shr, ShrString, Sram}, pdk::{Enviroment, Pdk}, simulate::{CircuitSimulator, ExecuteCommand, Meas}, YouRAMResult};
pub type Bits = Vec<bool>;

pub enum SramTransaction {
    Write { address: Bits, word: Bits },
    Read { address: Bits },
}

/// Generate SRAM transaction and meas in logic.
/// 
/// Call `simulate` method to run spice simulate and get meas result
/// 
/// Clock set: 
///
///   transaction 1
///         |
///         v
///         +---+   +---+   +---+
/// clk:    |   |   |   |   |   |
///     +---+   +---+   +---+   +---
///     ^       ^
///     |       |
///    t = 0   t = period
///
///
pub struct SramTransactionGenerator {
    pub sram: Shr<Sram>,
    pub period: Time,

    transactions: Vec<SramTransaction>,
    read_transaction_size: usize,
    write_transaction_size: usize,
    measurements: Vec<Box<dyn Meas>>, 
    memory: HashMap<usize, Vec<bool>>,
    addr_mask: usize,
    word_mask: usize,
    max_address: usize,
    max_word: usize,
}

impl SramTransactionGenerator {
    pub fn new(sram: Shr<Sram>, period: Time) -> Self {
        let addr_mask = Self::full_bits_number(sram.read().address_width());
        let word_mask = Self::full_bits_number(sram.read().word_width());
        let max_address = 2usize.pow(sram.read().address_width() as u32) - 1;
        let max_word = 2usize.pow(sram.read().word_width() as u32) - 1;
        
        Self {
            sram,
            period,
            transactions: vec![],
            read_transaction_size: 0,
            write_transaction_size: 0,
            measurements: vec![],
            memory: HashMap::new(),
            addr_mask,
            word_mask,
            max_address,
            max_word,
        }
    }

    pub fn simulate(
        self,
        env: Enviroment,
        pdk: Arc<Pdk>,
        execute: impl ExecuteCommand,
        simulate_path: impl AsRef<Path>,
        circuit_path: impl AsRef<Path>, 
        temp_folder: impl AsRef<Path>,
    ) -> YouRAMResult<HashMap<String, Number>>  {
        let mut simulator = CircuitSimulator::create(self.sram.clone(), env, pdk, simulate_path, circuit_path)?;
    
        // transform logic transactions to real voltages values
        let mut we_voltags = vec![];
        let mut address_voltags = vec![vec![]; self.sram.read().address_width()];
        let mut word_voltags = vec![vec![]; self.sram.read().word_width()];

        for transaction in self.transactions.iter() {
            match transaction {
                SramTransaction::Write { address, word } => {
                    we_voltags.push(simulator.logic1_voltage());
                    
                    for (voltags, &value) in address_voltags.iter_mut().zip(address) {
                        voltags.push(simulator.logic_voltage(value));
                    }

                    for (voltags, &value) in word_voltags.iter_mut().zip(word) {
                        voltags.push(simulator.logic_voltage(value));
                    }

                }
                SramTransaction::Read { address } => {
                    we_voltags.push(v!(0));
                    for (voltags, &value) in address_voltags.iter_mut().zip(address) {
                        voltags.push(simulator.logic_voltage(value));
                    }

                    for voltags in word_voltags.iter_mut() {
                        voltags.push(v!(0.));
                    }
                }
            }
        }

        // write inputs
        simulator.write_clock(self.period)?;

        let mut write_stimulation = |port_name: ShrString, voltages: &[Voltage]| -> YouRAMResult<()> {
            simulator.write_period_stimulate(port_name, voltages, self.period, 0.0)
        };
            
        write_stimulation(Sram::chip_sel_bar_pn(), &[v!(0.)])?;
        write_stimulation(Sram::write_enable_pn(), &we_voltags)?;
        
        for (i, address) in address_voltags.iter().enumerate() {
            write_stimulation(Sram::address_pn(i), address)?;
        }

        for (i, word) in word_voltags.iter().enumerate() {
            write_stimulation(Sram::data_input_pn(i), word)?;
        }

        // write meas
        for meas in self.measurements {
            simulator.write_measurement(meas)?;
        }

        // write trans
        let end_time = self.period * (self.transactions.len() + 2) as f64;
        simulator.write_trans(t!(10 p), 0.0, end_time)?;

        // run simulate
        simulator.simulate(execute, temp_folder)
    }

    pub fn add_random_write_transaction(&mut self) -> bool {
        let address = self.random_address();
        let word = self.random_word();
        self.add_write_transaction(address, word)
    }

    pub fn add_random_read_transaction(&mut self) -> bool {
        let address = self.random_address();
        self.add_read_transaction(address)
    }

    /// Add a read transaction, and update sram memory state
    pub fn add_write_transaction(&mut self, address: usize, word: usize) -> bool {
        let address: usize = self.mask_address(address);
        let word = self.mask_word(word);

        self.transactions.push(SramTransaction::write(
            self.address_to_bits(address), 
            self.word_to_bits(word)
        ));
        self.write_transaction_size += 1;

        self.memory.insert(address, self.word_to_bits(word));

        true
    }

    /// Add a read transaction
    /// if address not writed yet, return false
    pub fn add_read_transaction(&mut self, address: usize) -> bool {
        let address: usize = self.mask_address(address);

        if !self.memory.contains_key(&address) {
            warn!("try to read an unset address 0x{0:x}, this transaction will be ignored.", address);
            return false;
        }

        self.transactions.push(SramTransaction::read(self.address_to_bits(address)));
        self.read_transaction_size += 1;

        true
    } 

    pub fn add_measurement<M: Meas + 'static>(&mut self, meas: impl Into<Box<M>>) {
        self.measurements.push(meas.into());
    }

    pub fn clock_rise_time(&self, clock_index: usize) -> Time {
        clock_index as f64 * self.period + self.period / 2.
    }

    pub fn clock_begin(&self, clock_index: usize) -> Time {
        clock_index as f64 * self.period
    }

    pub fn read_transaction_size(&self) -> usize {
        self.read_transaction_size
    }

    pub fn write_transaction_size(&self) -> usize {
        self.write_transaction_size
    }

    pub fn memory(&self, address: usize) -> Option<&Bits> {
        self.memory.get(&address)
    }

    pub fn transaction_size(&self) -> usize {
        self.transactions.len()
    }

    #[inline]
    fn address_to_bits(&self, address: usize) -> Bits {
        Self::usize_to_bits(address, self.sram.read().args.address_width)
    }

    #[inline]
    fn word_to_bits(&self, word: usize) -> Bits {
        Self::usize_to_bits(word, self.sram.read().args.word_width)
    }

    #[inline]
    pub fn max_address(&self) -> usize {
        self.max_address
    }

    #[inline]
    pub fn max_word(&self) -> usize {
        self.max_word
    }

    #[inline]
    pub fn mask_address(&self, address: usize) -> usize {
        self.addr_mask & address
    }

    #[inline]
    pub fn mask_word(&self, word: usize) -> usize {
        self.word_mask & word
    }

    #[inline]
    pub fn random_address(&self) -> usize {
        Self::random_usize(self.max_address)  
    }

    #[inline]
    pub fn random_word(&self) -> usize {
        Self::random_usize(self.max_word)
    }

    /// generate a usize in range [0, max]
    fn random_usize(max: usize) -> usize {
        let mut rng = rand::rng();
        rng.random_range(0..=max)
    }

    fn usize_to_bits(mut value: usize, size: usize) -> Bits {
        let mut bits = vec![false; size];
        for i in 0..size {
            let bit = 0 != (value & 0x00000001usize);
            bits[i] = bit;
            value >>= 1;
        }

        bits
    }

    fn full_bits_number(size: usize) -> usize {
        let mut value = 0usize;
        for _ in 0..size {
            value = (value << 1) + 1;
        } 
        value
    }
    
}

impl SramTransaction {
    pub fn write(address: impl Into<Bits>, word: impl Into<Bits>) -> Self {
        Self::Write { address: address.into(), word: word.into() }
    }

    pub fn read(address: impl Into<Bits>) -> Self {
        Self::Read { address: address.into() }
    }
}
