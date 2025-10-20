use youram_macro::module;
use crate::{check_arg, circuit::{CircuitFactory, DriveStrength, ShrString, LogicGateKind}, format_shr, YouRAMResult};

const MAX_FANOUT_SIZE: usize = 1024;

#[module(
    input:  ("in", Input),
    output: ("out{fanout_size}", Output),
    vdd:    ("vdd", Source),
    gnd:    ("gnd", Source),
)]
pub struct FanoutBuffer {
    pub fanout_size: usize,
}

impl FanoutBuffer {
    pub fn build(&mut self, factory: &mut CircuitFactory) -> YouRAMResult<()> {
        check_arg!(self.args.fanout_size > 1, "Fanout width '{}' <= 1", self.args.fanout_size);

        let mut output_size = self.args.fanout_size;
        let mut tree_level_fanout_infos = vec![];
        loop {
            let info = Self::calculate_fanout_info(output_size);
            let inv_size = info.inv_size;
            output_size = inv_size;
            tree_level_fanout_infos.push(info);
            if inv_size == 1 {
                break;
            }
        }

        let inv = self.add_logicgate(LogicGateKind::Inv, DriveStrength::X2, factory)?;
        let tree_depth = tree_level_fanout_infos.len();

        let tree_level_inv_name = |level: usize, inv_index: usize| -> ShrString {
            format_shr!("inv_{}_{}", level, inv_index)
        };
        let tree_level_inv_output_name = |level: usize, inv_index: usize| -> ShrString {
            format_shr!("net_{}_{}", level, inv_index)
        };
        let tree_level_inv_input_name = |level: usize, inv_index: usize| -> ShrString {
            if level == 0 {
                return "net_begin".into();
            }
            let fanout = tree_level_fanout_infos[level - 1].fanout;
            tree_level_inv_output_name(level - 1, inv_index / fanout)
        };

        for level in 0..tree_depth {
            let inv_size = tree_level_fanout_infos[level].inv_size;

            // Now, this level has 'inputSize' input, each of input will generate 'fanout' output
            for inv_index in 0..inv_size {
                let name = tree_level_inv_name(level, inv_index);
                let input_net = tree_level_inv_input_name(level, inv_index);
                let output_net = tree_level_inv_output_name(level, inv_index);
                
                self.link_inv_instance(name, inv.clone(), [input_net, output_net, Self::vdd_pn(), Self::gnd_pn()])?;
            }
        }

        if tree_depth % 2 != 0 {
            self.link_inv_instance("inv", inv.clone(), [
                Self::input_pn(),
                tree_level_inv_input_name(0, 0),
                Self::vdd_pn(),
                Self::gnd_pn(),
            ])?;
        } else {
            self.connect_nets(Self::input_pn(), tree_level_inv_input_name(0, 0));
        }

        // connect output
        for output_index in 0..self.args.fanout_size {
            self.connect_nets(
                Self::output_pn(output_index),
                tree_level_inv_output_name(tree_depth, output_index)
            );
        }

        Ok(())
    }


    fn calculate_fanout_info(output_size: usize) -> FanoutInfo {
        if output_size <= MAX_FANOUT_SIZE {
            return FanoutInfo::new(1, output_size, 0);
        }

        let mut inv_size = 2;
        loop {
            let remainder = output_size % inv_size == 0;
            let fanout = output_size / inv_size + (if remainder { 0 } else { 1 });
            if fanout > MAX_FANOUT_SIZE {
                inv_size += 1;
                continue;
            }

            let delta = fanout * inv_size - output_size;
            return FanoutInfo::new(inv_size, fanout, delta);
        }
    }
}

#[derive(derive_new::new)]
struct FanoutInfo {
    inv_size: usize, 
    fanout: usize, 
    #[allow(unused)]
    delta: usize,
}
