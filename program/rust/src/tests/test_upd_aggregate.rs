use solana_program::pubkey::Pubkey;
use std::mem::size_of;

use crate::c_oracle_header::{
    cmd_upd_price_t,
    command_t_e_cmd_upd_price,
    pc_price_info_t,
    pc_price_t,
    PC_STATUS_TRADING,
    PC_STATUS_UNKNOWN,
    PC_VERSION,
};

use crate::deserialize::{
    initialize_pyth_account_checked,
    load_checked,
    load_mut,
};
use crate::rust_oracle::c_upd_aggregate;
use crate::tests::test_utils::AccountSetup;
#[test]
fn test_upd_aggregate() {
    let p1: pc_price_info_t = pc_price_info_t {
        price_:           100,
        conf_:            10,
        status_:          PC_STATUS_TRADING,
        pub_slot_:        1000,
        corp_act_status_: 0,
    };

    let p2: pc_price_info_t = pc_price_info_t {
        price_:           200,
        conf_:            20,
        status_:          PC_STATUS_TRADING,
        pub_slot_:        1000,
        corp_act_status_: 0,
    };

    let p3: pc_price_info_t = pc_price_info_t {
        price_:           300,
        conf_:            30,
        status_:          PC_STATUS_TRADING,
        pub_slot_:        1000,
        corp_act_status_: 0,
    };

    let p4: pc_price_info_t = pc_price_info_t {
        price_:           400,
        conf_:            40,
        status_:          PC_STATUS_TRADING,
        pub_slot_:        1000,
        corp_act_status_: 0,
    };

    let mut instruction_data = [0u8; size_of::<cmd_upd_price_t>()];
    populate_instruction(&mut instruction_data, 42, 2, 1);

    let program_id = Pubkey::new_unique();

    let mut price_setup = AccountSetup::new::<pc_price_t>(&program_id);
    let mut price_account = price_setup.to_account_info();
    price_account.is_signer = false;
    initialize_pyth_account_checked::<pc_price_t>(&price_account, PC_VERSION).unwrap();

    // single publisher
    {
        let mut price_data = load_checked::<pc_price_t>(&price_account, PC_VERSION).unwrap();
        price_data.num_ = 1;
        price_data.last_slot_ = 1000;
        price_data.agg_.pub_slot_ = 1000;
        price_data.comp_[0].latest_ = p1;
    }
    unsafe {
        assert!(c_upd_aggregate(
            price_account.try_borrow_mut_data().unwrap().as_mut_ptr(),
            1001,
            1,
        ));
    }

    {
        let price_data = load_checked::<pc_price_t>(&price_account, PC_VERSION).unwrap();

        assert_eq!(price_data.agg_.price_, 100);
        assert_eq!(price_data.agg_.conf_, 10);
        assert_eq!(price_data.twap_.val_, 100);
        assert_eq!(price_data.twac_.val_, 10);
        assert_eq!(price_data.num_qt_, 1);
        assert_eq!(price_data.timestamp_, 1);
        assert_eq!(price_data.prev_slot_, 0);
        assert_eq!(price_data.prev_price_, 0);
        assert_eq!(price_data.prev_conf_, 0);
        assert_eq!(price_data.prev_timestamp_, 0);
    }

    // two publishers
    {
        let mut price_data = load_checked::<pc_price_t>(&price_account, PC_VERSION).unwrap();

        price_data.num_ = 2;
        price_data.last_slot_ = 1000;
        price_data.agg_.pub_slot_ = 1000;
        price_data.comp_[0].latest_ = p1;
        price_data.comp_[1].latest_ = p2;
    }

    unsafe {
        assert!(c_upd_aggregate(
            price_account.try_borrow_mut_data().unwrap().as_mut_ptr(),
            1001,
            2,
        ));
    }

    {
        let price_data = load_checked::<pc_price_t>(&price_account, PC_VERSION).unwrap();

        assert_eq!(price_data.agg_.price_, 145);
        assert_eq!(price_data.agg_.conf_, 55);
        assert_eq!(price_data.twap_.val_, 106);
        assert_eq!(price_data.twac_.val_, 16);
        assert_eq!(price_data.num_qt_, 2);
        assert_eq!(price_data.timestamp_, 2);
        assert_eq!(price_data.prev_slot_, 1000);
        assert_eq!(price_data.prev_price_, 100);
        assert_eq!(price_data.prev_conf_, 10);
        assert_eq!(price_data.prev_timestamp_, 1);
    }


    // three publishers
    {
        let mut price_data = load_checked::<pc_price_t>(&price_account, PC_VERSION).unwrap();

        price_data.num_ = 3;
        price_data.last_slot_ = 1000;
        price_data.agg_.pub_slot_ = 1000;
        price_data.comp_[0].latest_ = p1;
        price_data.comp_[1].latest_ = p2;
        price_data.comp_[2].latest_ = p3;
    }

    unsafe {
        assert!(c_upd_aggregate(
            price_account.try_borrow_mut_data().unwrap().as_mut_ptr(),
            1001,
            3,
        ));
    }

    {
        let price_data = load_checked::<pc_price_t>(&price_account, PC_VERSION).unwrap();

        assert_eq!(price_data.agg_.price_, 200);
        assert_eq!(price_data.agg_.conf_, 90);
        assert_eq!(price_data.twap_.val_, 114);
        assert_eq!(price_data.twac_.val_, 23);
        assert_eq!(price_data.num_qt_, 3);
        assert_eq!(price_data.timestamp_, 3);
        assert_eq!(price_data.prev_slot_, 1000);
        assert_eq!(price_data.prev_price_, 145);
        assert_eq!(price_data.prev_conf_, 55);
        assert_eq!(price_data.prev_timestamp_, 2);
    }

    // four publishers
    {
        let mut price_data = load_checked::<pc_price_t>(&price_account, PC_VERSION).unwrap();

        price_data.num_ = 4;
        price_data.last_slot_ = 1000;
        price_data.agg_.pub_slot_ = 1000;
        price_data.comp_[0].latest_ = p1;
        price_data.comp_[1].latest_ = p2;
        price_data.comp_[2].latest_ = p3;
        price_data.comp_[3].latest_ = p4;
    }

    unsafe {
        assert!(c_upd_aggregate(
            price_account.try_borrow_mut_data().unwrap().as_mut_ptr(),
            1001,
            4,
        ));
    }

    {
        let price_data = load_checked::<pc_price_t>(&price_account, PC_VERSION).unwrap();

        assert_eq!(price_data.agg_.price_, 245);
        assert_eq!(price_data.agg_.conf_, 85);
        assert_eq!(price_data.twap_.val_, 125);
        assert_eq!(price_data.twac_.val_, 28);
        assert_eq!(price_data.num_qt_, 4);
        assert_eq!(price_data.timestamp_, 4);
        assert_eq!(price_data.last_slot_, 1001);
        assert_eq!(price_data.prev_slot_, 1000);
        assert_eq!(price_data.prev_price_, 200);
        assert_eq!(price_data.prev_conf_, 90);
        assert_eq!(price_data.prev_timestamp_, 3);
    }

    unsafe {
        assert!(c_upd_aggregate(
            price_account.try_borrow_mut_data().unwrap().as_mut_ptr(),
            1025,
            5,
        ));
    }

    {
        let price_data = load_checked::<pc_price_t>(&price_account, PC_VERSION).unwrap();

        assert_eq!(price_data.agg_.status_, PC_STATUS_TRADING);
        assert_eq!(price_data.last_slot_, 1025);
        assert_eq!(price_data.num_qt_, 4);
        assert_eq!(price_data.timestamp_, 5);
        assert_eq!(price_data.prev_slot_, 1001);
        assert_eq!(price_data.prev_price_, 245);
        assert_eq!(price_data.prev_conf_, 85);
        assert_eq!(price_data.prev_timestamp_, 4);
    }

    // check what happens when nothing publishes for a while
    unsafe {
        assert!(!c_upd_aggregate(
            price_account.try_borrow_mut_data().unwrap().as_mut_ptr(),
            1026,
            10,
        ));
    }
    {
        let price_data = load_checked::<pc_price_t>(&price_account, PC_VERSION).unwrap();

        assert_eq!(price_data.agg_.status_, PC_STATUS_UNKNOWN);
        assert_eq!(price_data.last_slot_, 1025);
        assert_eq!(price_data.num_qt_, 0);
        assert_eq!(price_data.timestamp_, 10);
        assert_eq!(price_data.prev_slot_, 1025);
        assert_eq!(price_data.prev_price_, 245);
        assert_eq!(price_data.prev_conf_, 85);
        assert_eq!(price_data.prev_timestamp_, 5);
    }

    unsafe {
        assert!(!c_upd_aggregate(
            price_account.try_borrow_mut_data().unwrap().as_mut_ptr(),
            1028,
            12,
        ));
    }

    {
        let price_data = load_checked::<pc_price_t>(&price_account, PC_VERSION).unwrap();

        assert_eq!(price_data.agg_.status_, PC_STATUS_UNKNOWN);
        assert_eq!(price_data.last_slot_, 1025);
        assert_eq!(price_data.num_qt_, 0);
        assert_eq!(price_data.timestamp_, 12);
        assert_eq!(price_data.prev_slot_, 1025);
        assert_eq!(price_data.prev_price_, 245);
        assert_eq!(price_data.prev_conf_, 85);
        assert_eq!(price_data.prev_timestamp_, 5);
    }
}

// Create an upd_price instruction with the provided parameters
fn populate_instruction(instruction_data: &mut [u8], price: i64, conf: u64, pub_slot: u64) -> () {
    let mut cmd = load_mut::<cmd_upd_price_t>(instruction_data).unwrap();
    cmd.ver_ = PC_VERSION;
    cmd.cmd_ = command_t_e_cmd_upd_price as i32;
    cmd.status_ = PC_STATUS_TRADING;
    cmd.price_ = price;
    cmd.conf_ = conf;
    cmd.pub_slot_ = pub_slot;
    cmd.unused_ = 0;
}
