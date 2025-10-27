use std::collections::HashSet;
use crate::YouRAMResult;
use super::{FunctionCharzPolicy, FunctionTransactionGenerator};
use rand::Rng;
use tracing::debug;

pub struct RandomPolicy;

impl FunctionCharzPolicy for RandomPolicy {
    /*

        1. 生成 N 个地址

        2. 先给 N 个地址写入初始值，并且在写入一个地址后马上插入对应的读操作！

        3. 开始随机产生读/写操作，但读信号地址必须来自这 N 个地址，写信号可以不需要

        在产生至少 2*N 个读操作后结束。

    */
    fn generate_transactions(&self, charz: &mut FunctionTransactionGenerator) -> YouRAMResult<()> {
        debug!("generate transactions with random policy");
        let read_transaction_size = 1.max(( 0.1 * charz.transactions.sram.read().word_size() as f64 ) as usize);
        let addresses = self.generate_random_address(charz, read_transaction_size)?;

        // 先给 N 个地址写入初始值，并且在写入一个地址后马上插入对应的读操作！
        for &address in addresses.iter() {
            charz.add_write_transaction(address, charz.transactions.random_word());
            charz.add_read_transaction(address);
        }

        // 开始随机产生读/写操作，但读信号地址必须来自这 N 个地址，写信号可以不需要
        let mut rng = rand::rng();
        let target_size = 2usize.pow(read_transaction_size as u32);
        while charz.transactions.read_transaction_size() <= target_size {
            let is_write: bool = rng.random_bool(0.5);
            if is_write {
                let address = charz.transactions.random_address();
                let word = charz.transactions.random_word();
                charz.add_write_transaction(address, word);
            } else {
                let address_index = rng.random_range(0..addresses.len());
                assert!(address_index < addresses.len());
                let address = addresses[address_index];
                charz.add_read_transaction(address);
            }
        }

        Ok(())
    }
}

impl RandomPolicy {
    fn generate_random_address(&self, charz: &mut FunctionTransactionGenerator, read_transaction_size: usize) -> YouRAMResult<Vec<usize>> {
        let mut address_set = HashSet::new();
        while address_set.len() < read_transaction_size {
            let address = charz.transactions.random_address();
            address_set.insert(address);
        }

        Ok(address_set.into_iter().collect())
    }
}