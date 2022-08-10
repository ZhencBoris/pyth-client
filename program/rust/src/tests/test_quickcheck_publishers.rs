use std::collections::HashSet;
use std::mem::size_of;

use crate::c_oracle_header::{
    cmd_del_publisher,
    command_t_e_cmd_del_publisher,
    pc_price_comp,
    pc_price_t,
    pc_pub_key_t,
    PythAccount,
    PC_COMP_SIZE,
    PC_VERSION,
};
use crate::deserialize::load_mut;
use crate::rust_oracle::{
    add_publisher,
    del_publisher,
    initialize_checked,
    load_checked,
};
use crate::tests::test_utils::AccountSetup;
use quickcheck::{
    Arbitrary,
    Gen,
};
use quickcheck_macros::quickcheck;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use std::fmt::{
    Debug,
    Formatter,
};


#[derive(Clone, Debug)]
enum Operation {
    Add(pc_pub_key_t),
    Delete(usize),
}

impl Debug for pc_pub_key_t {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        unsafe { return self.k8_.fmt(f) }
    }
}

impl pc_price_t {
    fn to_set(&self) -> HashSet<[u64; 4]> {
        let mut res: HashSet<[u64; 4]> = HashSet::new();
        for i in 0..(self.num_ as usize) {
            unsafe { res.insert(self.comp_[i].pub_.k8_) };
        }
        return res;
    }
}

impl Arbitrary for pc_pub_key_t {
    fn arbitrary(g: &mut Gen) -> Self {
        let mut array = [0u64; 4];
        for i in 0..4 {
            array[i] = u64::arbitrary(g);
        }
        return pc_pub_key_t { k8_: array };
    }
}

impl Arbitrary for Operation {
    fn arbitrary(g: &mut Gen) -> Self {
        let sample = u8::arbitrary(g);
        match sample % 2 {
            0 => {
                return Operation::Add(pc_pub_key_t::arbitrary(g));
            }
            1 => {
                let mut pos = u8::arbitrary(g);
                // Reroll until pos < PC_COMP_SIZE
                while pos >= PC_COMP_SIZE.try_into().unwrap() {
                    pos = u8::arbitrary(g);
                }
                return Operation::Delete(pos as usize);
            }
            _ => panic!(),
        }
    }
}

fn populate_data(instruction_data: &mut [u8], publisher: pc_pub_key_t) {
    let mut hdr = load_mut::<cmd_del_publisher>(instruction_data).unwrap();
    hdr.ver_ = PC_VERSION;
    hdr.cmd_ = command_t_e_cmd_del_publisher as i32;
    hdr.pub_ = publisher;
}

/// This quickcheck test will generate a series of Vec<Operation>. For each vector
/// each operation can be adding a publisher `pub_`
/// or deleting the publisher at position `i` (or a random publisher if `i > price_data.num_`)
/// A set is maintained with the publishers that should be authorized and the onchain struct
/// and the set are compared after each operation.
#[quickcheck]
fn test_add_and_delete(input: Vec<Operation>) -> bool {
    let mut set: HashSet<[u64; 4]> = HashSet::new();

    let program_id = Pubkey::new_unique();

    let mut funding_setup = AccountSetup::new_funding();
    let funding_account = funding_setup.to_account_info();

    let mut instruction_data = [0u8; size_of::<cmd_del_publisher>()];

    let mut price_setup = AccountSetup::new::<pc_price_t>(&program_id);
    let price_account = price_setup.to_account_info();
    initialize_checked::<pc_price_t>(&price_account, PC_VERSION).unwrap();


    for op in input {
        match op {
            // Add the given pubkey
            Operation::Add(pub_) => {
                populate_data(&mut instruction_data, pub_);

                match add_publisher(
                    &program_id,
                    &[funding_account.clone(), price_account.clone()],
                    &instruction_data,
                ) {
                    Ok(_) => {
                        unsafe { set.insert(pub_.k8_) };
                    }
                    Err(err) => {
                        assert_eq!(err, ProgramError::InvalidArgument);
                        let price_data =
                            load_checked::<pc_price_t>(&price_account, PC_VERSION).unwrap();
                        assert_eq!(price_data.num_, PC_COMP_SIZE);
                    }
                }
            }
            // Delete the publisher at position pos
            Operation::Delete(pos) => {
                let pubkey_to_delete: pc_pub_key_t;
                {
                    let price_data =
                        load_checked::<pc_price_t>(&price_account, PC_VERSION).unwrap();

                    if price_data.num_ as usize > pos {
                        pubkey_to_delete = price_data.comp_[pos].pub_; // If within range, delete
                                                                       // that publisher
                    } else {
                        pubkey_to_delete = pc_pub_key_t::new_unique(); // Else try deleting a
                                                                       // publisher that's not in
                                                                       // the array
                    }
                    populate_data(&mut instruction_data, pubkey_to_delete);
                }

                match del_publisher(
                    &program_id,
                    &[funding_account.clone(), price_account.clone()],
                    &instruction_data,
                ) {
                    Ok(_) => {
                        unsafe { set.remove(&pubkey_to_delete.k8_) };
                    }
                    Err(err) => {
                        assert_eq!(err, ProgramError::InvalidArgument);
                        let price_data =
                            load_checked::<pc_price_t>(&price_account, PC_VERSION).unwrap();
                        assert!(price_data.num_ as usize <= pos);
                    }
                }
            }
        }
        {
            let price_data = load_checked::<pc_price_t>(&price_account, PC_VERSION).unwrap();
            assert_eq!(price_data.to_set(), set);
            assert_eq!(
                price_data.size_ as usize,
                (pc_price_t::INITIAL_SIZE as usize) + set.len() * size_of::<pc_price_comp>()
            )
        }
    }

    return true;
}
