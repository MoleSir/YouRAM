use youram_macro::module;
use crate::{check_arg, circuit::{CircuitFactory, DriveStrength, LogicGateKind}, format_shr, ErrorContext, YouRAMResult};

const MAX_SIMPLE_INPUT_SIZE: usize = 4;
const MIN_INPUT_SIZE: usize = 1;
const MAX_INPUT_SIZE: usize = 12;
const SUB_DECODERS_INPUT_SIZES: [&'static [usize]; 8] = [
    &[2, 3], 
    &[3, 3], 
    &[3, 4], 
    &[4, 4],
    &[3, 3, 3],
    &[3, 3, 4],
    &[3, 4, 4],
    &[4, 4, 4],
];

#[module(
    address: ("A{input_size}", Input),
    output:  ("Y{output_size}", Output),
    vdd:     ("vdd", Vdd),
    gnd:     ("gnd", Gnd),
)]
pub struct Decoder {
    pub input_size: usize,

    #[new(value = "2usize.pow(input_size as u32)")]
    pub output_size: usize,
}

impl Decoder {
    pub fn build(&mut self, factory: &mut CircuitFactory) -> YouRAMResult<()> {
        check_arg!(self.args.input_size >= MIN_INPUT_SIZE, "Input size '{}' < {}", self.args.input_size, MIN_INPUT_SIZE);
        check_arg!(self.args.input_size <= MAX_INPUT_SIZE, "Input size '{}' > {}", self.args.input_size, MAX_INPUT_SIZE);

        match self.args.kind() {
            DecoderType::OneAddr => self.build_one_addr(factory),
            DecoderType::Simple => self.build_simple(factory),
            DecoderType::Componet => self.build_componet(factory),
        }

    }

    fn build_one_addr(&mut self, factory: &mut CircuitFactory) -> YouRAMResult<()> {
        let inv = self.add_logicgate(LogicGateKind::Inv, DriveStrength::X2, factory)?;
        
        self.link_inv_instance("inv0", inv.clone(), [Self::address_pn(0), Self::output_pn(0), Self::vdd_pn(), Self::gnd_pn()])?;
        self.link_inv_instance("inv1", inv.clone(), [Self::output_pn(0), Self::output_pn(1), Self::vdd_pn(), Self::gnd_pn()])?;

        Ok(())
    }

    fn build_simple(&mut self, factory: &mut CircuitFactory) -> YouRAMResult<()> {
        let inv = self.add_logicgate(LogicGateKind::Inv, DriveStrength::X2, factory)?;
        let and = self.add_logicgate(LogicGateKind::And(self.args.input_size), DriveStrength::X2, factory)?;
                        
        let input_ports: Vec<_> = (0..self.args.input_size).map(|i| Self::address_pn(i)).collect();
        let input_ports_bar: Vec<_> = (0..self.args.input_size).map(|i| format_shr!("A{}_bar", i)).collect();

        for i in 0..self.args.input_size {
            let inst_name = format!("inv{}", i);
            self.link_inv_instance(
                inst_name, inv.clone(), 
                [input_ports[i].clone(), input_ports_bar[i].clone(), Self::vdd_pn(), Self::gnd_pn()]
            )?;
        } 

        for i in 0..self.args.output_size {
            let mut input_nets = vec![];
            // 'i' is the AND gate's index. Each AND gate's inputs are [A0/A0_int, A1/A1_int ... An/An_int]
            // There are '_inputSize' inputs, and 'j' is the input port' index.
            // No.'j' bit in 'i' decides the port for Aj is inverted or not.
            // For example, i == 000, the inputs are [A0_int, A1_int, A2_int].
            //              i == 010, the inputs are [A0_int, A1,     A2_int].
            // ...
            for j in 0..self.args.input_size {
                let bit_one = ((i >> j) & 0x1) != 0;
                input_nets.push( if bit_one { input_ports[j].clone() } else { input_ports_bar[j].clone() } );   
            }

            let inst_name = format!("and{}", i);
            self.link_logicgate_instance(
                inst_name, and.clone(), 
                input_nets, Self::output_pn(i), Self::vdd_pn(), Self::gnd_pn()
            )?;
        }

        Ok(())
    }

    fn build_componet(&mut self, factory: &mut CircuitFactory) -> YouRAMResult<()> {
        let sub_decoders_input_size = self.sub_decoders_input_size();
        let and = self.add_logicgate(LogicGateKind::And(sub_decoders_input_size.len()), DriveStrength::X2, factory).context("create and")?;

        let mut global_input_index = 0;
        for (decoder_index, &sub_input_size) in sub_decoders_input_size.iter().enumerate() {
            let arg = DecoderArg::new(sub_input_size);
            let sub_decoder = self.add_module(arg, factory)?;

            // add nets
            let mut nets = vec![];
            for _ in 0..sub_input_size {
                nets.push(Self::address_pn(global_input_index));
                global_input_index += 1;
            }
            for ouput_index in 0..2usize.pow(sub_input_size as u32) { 
                nets.push(format_shr!("Y_{}_{}", decoder_index, ouput_index));
            }
            nets.push(Self::vdd_pn());
            nets.push(Self::gnd_pn());

            let inst_name = format!("decoder{}", decoder_index);
            let instance = self.add_instance(inst_name.clone(), sub_decoder)?;
            self.connect_instance(instance, nets.into_iter())?;
        }

        // for output AND
        for and_index in 0..self.args.output_size {
            let mut input_nets = vec![];
            // For each decoder, get an output line as AND gate's input
            // emmm... this algo is hard to explain, so TODO!!!
            for (decoder_index, &_) in sub_decoders_input_size.iter().enumerate() {
                let mut prefix_sum = 0;
                for index in 0..decoder_index {
                    prefix_sum += sub_decoders_input_size[index];
                }
                
                let mut mask = 1;
                for _ in 1..sub_decoders_input_size[decoder_index] {
                    mask = (mask << 1) + 1;
                }

                let decoder_output_index = (and_index >> prefix_sum) & mask;
                input_nets.push(format_shr!("Y_{}_{}", decoder_index, decoder_output_index));
            }   

            self.link_logicgate_instance(format!("and{}", and_index), and.clone(), 
                input_nets, Self::output_pn(and_index), Self::vdd_pn(), Self::gnd_pn()
            )?;
        }

        Ok(())
    }

    fn sub_decoders_index(&self) -> usize {
        self.args.input_size - 1 - MAX_SIMPLE_INPUT_SIZE
    } 

    fn sub_decoders_input_size(&self) -> &'static [usize] {
        SUB_DECODERS_INPUT_SIZES[self.sub_decoders_index()]
    }
}

pub enum DecoderType {
    OneAddr,
    Simple,
    Componet,
}

impl DecoderArg {
    pub fn kind(&self) -> DecoderType {
        match self.input_size {
            1 => DecoderType::OneAddr,
            i if i <= MAX_SIMPLE_INPUT_SIZE => DecoderType::Simple,
            _ => DecoderType::Componet,
        }
    }
}