use tracing::info;
use youram_macro::module;
use crate::{circuit::CircuitFactory, format_shr, YouRAMResult};
use super::{Core, CoreArg, CoreSelector, CoreSelectorArg, DecoderArg, InputDffs, InputDffsArg};

#[module(
    clock:         ("clk", Input),
    chip_sel_bar:  ("csb", Input),
    write_enable:  ("we", Input),

    address:       ("addr{address_width}", Input),
    data_input:    ("din{word_width}", Input),
    data_output:   ("dout{word_width}", Input),

    vdd:           ("vdd", Source),
    gnd:           ("gnd", Source),
)]
pub struct Sram {
    pub address_width: usize,
    pub word_width: usize,

    #[new(value = "AddressDistribution::new(address_width, word_width)")]
    pub distribution: AddressDistribution
}

impl Sram {
    const MAX_CORE_ADDRESS_WIDTH: usize = 2;
    const MAX_CORE_SIZE: usize = 2usize.pow(Self::MAX_CORE_ADDRESS_WIDTH as u32);
    const MAX_COLUMN_ADDRESS_WIDTH: usize = 3;
    const MAX_BITCELL_SIZE: usize = Self::MAX_CORE_SIZE * Core::MAX_BITCELL_SIZE;

    pub fn build(&mut self, factory: &mut CircuitFactory) -> YouRAMResult<()> {
        info!("address distribution: {:?}", self.args.distribution);
        // add module
        let input_dffs 
            = self.add_module(InputDffsArg::new(self.args.address_width, self.args.word_width), factory)?;
        let core 
            = self.add_module(CoreArg::new(self.core_row_size(), self.core_column_sel_size(), self.args.word_width), factory)?;
        let row_decoder
            = self.add_module(DecoderArg::new(self.row_address_width()), factory)?;
        let column_decoder = if self.column_address_width() > 0 {
            Some(self.add_module(DecoderArg::new(self.column_address_width()), factory)?)
        } else {
            None
        };

        // golbal nets
        let addr_reg_nets: Vec<_> = (0..self.args.address_width).map(|i| format_shr!("addr{}_r", i)).collect();
        let addr_nets: Vec<_> = (0..self.args.address_width).map(|i| Self::address_pn(i)).collect();

        let din_reg_nets: Vec<_> = (0..self.args.word_width).map(|i| format_shr!("din{}_r", i)).collect();
        let din_nets: Vec<_> = (0..self.args.word_width).map(|i| Self::data_input_pn(i)).collect();        
        
        let cbs_r_net = InputDffs::chip_sel_bar_reg_pn();
        let we_r_net = InputDffs::write_enable_reg_pn();
        
        // input dff
        {
            let mut nets = vec![
                Self::clock_pn(),
                Self::chip_sel_bar_pn(),
                Self::write_enable_pn()
            ];
            nets.extend(addr_nets.iter().cloned());
            nets.extend(din_nets.iter().cloned());

            nets.push(cbs_r_net.clone());
            nets.push(we_r_net.clone());
            nets.extend(addr_reg_nets.iter().cloned());
            nets.extend(din_reg_nets.iter().cloned());

            nets.push(Self::vdd_pn());
            nets.push(Self::gnd_pn());
        
            self.link_module_instance("input_dffs", input_dffs, nets.into_iter())?;
        }

        // row address decoder
        let rsel_nets: Vec<_> = (0..self.core_row_size()).map(|i| format_shr!("rsel{}", i)).collect();
        {
            let mut nets = vec![];
            nets.extend((0..self.row_address_width()).map(|i| addr_reg_nets[i+self.column_address_width()].clone()));
            nets.extend(rsel_nets.iter().cloned());
            nets.push(Self::vdd_pn());
            nets.push(Self::gnd_pn());

            self.link_module_instance("row_decoder", row_decoder, nets.into_iter())?;
        }

        // col address decoder
        let csel_nets = if let Some(col_decoder) = column_decoder.as_ref() {
            let csel_nets: Vec<_> = (0..self.core_column_sel_size()).map(|i| format_shr!("csel{}", i)).collect();
            let mut nets = vec![];
            nets.extend((0..self.column_address_width()).map(|i| addr_reg_nets[i].clone()));
            nets.extend(csel_nets.iter().cloned());
            nets.push(Self::vdd_pn());
            nets.push(Self::gnd_pn());

            self.link_module_instance("col_decoder", col_decoder.clone(), nets.into_iter())?;
            csel_nets
        } else {
            vec![]
        };

        // Sram core
        if self.multiple_core() {
            let core_sel
                = self.add_module(CoreSelectorArg::new(self.core_address_width(), self.args.word_width), factory)?;

            let core_csb_nets: Vec<_> = (0..self.core_count()).map(|c| CoreSelector::chip_sel_bar_core_pn(c)).collect();
            let core_dout_nets = (0..self.core_count()).map(|core| {
                (0..self.args.word_width).map(move |bit| CoreSelector::data_output_core_pn(core, bit)).collect::<Vec<_>>()
            }).collect::<Vec<_>>();

            // core select
            {
                let mut nets = vec![];
                nets.push(cbs_r_net.clone());
                nets.extend((0..self.core_address_width()).map(|i| addr_reg_nets[i + self.column_address_width() + self.row_address_width()].clone()));
                nets.extend(core_dout_nets.iter().flatten().cloned());
                nets.extend(core_csb_nets.iter().cloned());
                nets.extend((0..self.args.word_width).map(|bit| Self::data_output_pn(bit)));
                nets.push(Self::vdd_pn());
                nets.push(Self::gnd_pn());

                self.link_module_instance("core_selector", core_sel, nets.into_iter())?;
            }

            // for each core
            for core_index in 0..self.core_count() {
                let mut nets = vec![];
                nets.push(Self::clock_pn());
                nets.push(core_csb_nets[core_index].clone());
                nets.push(we_r_net.clone());

                nets.extend(rsel_nets.iter().cloned());
                nets.extend(csel_nets.iter().cloned());

                nets.extend(din_reg_nets.iter().cloned());
                nets.extend(core_dout_nets[core_index].iter().cloned());

                nets.push(Self::vdd_pn());
                nets.push(Self::gnd_pn());

                self.link_module_instance(format_shr!("core{}", core_index), core.clone(), nets.into_iter())?;
            }

        } else {
            let mut nets = vec![];
            nets.push(Self::clock_pn());
            nets.push(cbs_r_net.clone());
            nets.push(we_r_net.clone());

            nets.extend(rsel_nets.iter().cloned());
            nets.extend(csel_nets.iter().cloned());

            nets.extend(din_reg_nets.iter().cloned());
            nets.extend((0..self.args.word_width).map(|bit| Self::data_output_pn(bit)));

            nets.push(Self::vdd_pn());
            nets.push(Self::gnd_pn());

            self.link_module_instance("core", core.clone(), nets.into_iter())?;            
        }

        Ok(())
    }

    pub fn core_count(&self) -> usize {
        2usize.pow(self.core_address_width() as u32)
    }

    pub fn core_row_size(&self) -> usize {
        2usize.pow(self.row_address_width() as u32)
    }

    pub fn core_column_sel_size(&self) -> usize {
        2usize.pow(self.column_address_width() as u32)
    }

    pub fn core_column_size(&self) -> usize {
        2usize.pow(self.column_address_width() as u32) * self.args.word_width
    }

    pub fn multiple_core(&self) -> bool {
        self.core_address_width() > 0
    }

    pub fn core_address_width(&self) -> usize {
        self.args.distribution.core_address_width
    }

    pub fn column_address_width(&self) -> usize {
        self.args.distribution.column_address_width
    }

    pub fn row_address_width(&self) -> usize {
        self.args.distribution.row_address_width
    }
}

#[derive(Debug)]
pub struct AddressDistribution {
    pub core_address_width: usize,
    pub row_address_width: usize,
    pub column_address_width: usize,
}

impl AddressDistribution {
    pub fn new(address_width: usize, word_width: usize) -> Self {
        let total_bits = 2usize.pow(address_width as u32) * word_width;
        assert!(total_bits <= Sram::MAX_BITCELL_SIZE, "Bit-cell size '{}' out of range '{}'", total_bits, Sram::MAX_BITCELL_SIZE);

        for core_address_width in 0..=Sram::MAX_CORE_ADDRESS_WIDTH {
            if let Some(column_address_width) = Self::try_one_core(address_width - core_address_width, word_width) {
                return Self { 
                    core_address_width,
                    column_address_width,
                    row_address_width: address_width - column_address_width - core_address_width
                };
            }
        }

        panic!("Can't find valid address distribution for the option of (address width: {}, word width: {})", address_width, word_width);
    }

    fn try_one_core(address_width: usize, word_width: usize) -> Option<usize> {
        let mut array_config = vec![];
        let max_col_address = Sram::MAX_COLUMN_ADDRESS_WIDTH.min(address_width - 1);

        // Generate all possible configs
        for col_addr_width in 0..=max_col_address {
            let row = 2usize.pow((address_width - col_addr_width) as u32);
            let col = word_width * 2usize.pow(col_addr_width as u32);
            let delta = if row > col { row - col } else { col - row };
            array_config.push((row, col, col_addr_width, delta));
        }  

        // Sort by delta 
        array_config.sort_by(|left, right| left.3.cmp(&right.3));

        // Find first config with satisfy constraints
        for (row, col, col_addr_width, _) in array_config {
            if row <= Core::MAX_ROW_SIZE && col <= Core::MAX_COLUMN_SIZE {
                return Some(col_addr_width);
            }
        }

        None
    }
}
