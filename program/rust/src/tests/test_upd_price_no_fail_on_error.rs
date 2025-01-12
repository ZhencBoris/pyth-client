use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use std::mem::size_of;

use crate::c_oracle_header::{
    cmd_upd_price_t,
    command_t_e_cmd_upd_price_no_fail_on_error,
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
use crate::rust_oracle::{
    upd_price,
    upd_price_no_fail_on_error,
};
use crate::tests::test_utils::{
    update_clock_slot,
    AccountSetup,
};
use crate::utils::pubkey_assign;
#[test]
fn test_upd_price_no_fail_on_error_no_fail_on_error() {
    let mut instruction_data = [0u8; size_of::<cmd_upd_price_t>()];

    let program_id = Pubkey::new_unique();

    let mut funding_setup = AccountSetup::new_funding();
    let funding_account = funding_setup.to_account_info();

    let mut price_setup = AccountSetup::new::<pc_price_t>(&program_id);
    let mut price_account = price_setup.to_account_info();
    price_account.is_signer = false;
    initialize_pyth_account_checked::<pc_price_t>(&price_account, PC_VERSION).unwrap();

    let mut clock_setup = AccountSetup::new_clock();
    let mut clock_account = clock_setup.to_account_info();
    clock_account.is_signer = false;
    clock_account.is_writable = false;

    update_clock_slot(&mut clock_account, 1);
    populate_instruction(&mut instruction_data, 42, 9, 1);


    // Check that the normal upd_price fails
    assert_eq!(
        upd_price(
            &program_id,
            &[
                funding_account.clone(),
                price_account.clone(),
                clock_account.clone()
            ],
            &instruction_data
        ),
        Err(ProgramError::InvalidArgument)
    );


    // We haven't permissioned the publish account for the price account
    // yet, so any update should fail silently and have no effect. The
    // transaction should "succeed".
    assert!(upd_price_no_fail_on_error(
        &program_id,
        &[
            funding_account.clone(),
            price_account.clone(),
            clock_account.clone()
        ],
        &instruction_data
    )
    .is_ok());


    {
        let mut price_data = load_checked::<pc_price_t>(&price_account, PC_VERSION).unwrap();
        assert_eq!(price_data.comp_[0].latest_.price_, 0);
        assert_eq!(price_data.comp_[0].latest_.conf_, 0);
        assert_eq!(price_data.comp_[0].latest_.pub_slot_, 0);
        assert_eq!(price_data.comp_[0].latest_.status_, PC_STATUS_UNKNOWN);
        assert_eq!(price_data.valid_slot_, 0);
        assert_eq!(price_data.agg_.pub_slot_, 0);
        assert_eq!(price_data.agg_.price_, 0);
        assert_eq!(price_data.agg_.status_, PC_STATUS_UNKNOWN);

        // Now permission the publish account for the price account.
        price_data.num_ = 1;
        pubkey_assign(
            &mut price_data.comp_[0].pub_,
            &funding_account.key.to_bytes(),
        );
    }

    // The update should now succeed, and have an effect.
    assert!(upd_price_no_fail_on_error(
        &program_id,
        &[
            funding_account.clone(),
            price_account.clone(),
            clock_account.clone()
        ],
        &instruction_data
    )
    .is_ok());

    {
        let price_data = load_checked::<pc_price_t>(&price_account, PC_VERSION).unwrap();
        assert_eq!(price_data.comp_[0].latest_.price_, 42);
        assert_eq!(price_data.comp_[0].latest_.conf_, 9);
        assert_eq!(price_data.comp_[0].latest_.pub_slot_, 1);
        assert_eq!(price_data.comp_[0].latest_.status_, PC_STATUS_UNKNOWN);
        assert_eq!(price_data.valid_slot_, 0);
        assert_eq!(price_data.agg_.pub_slot_, 1);
        assert_eq!(price_data.agg_.price_, 0);
        assert_eq!(price_data.agg_.status_, PC_STATUS_UNKNOWN);
    }

    // Invalid updates, such as publishing an update for the current slot,
    // should still fail silently and have no effect.
    populate_instruction(&mut instruction_data, 55, 22, 1);

    // Check that the normal upd_price fails
    assert_eq!(
        upd_price(
            &program_id,
            &[
                funding_account.clone(),
                price_account.clone(),
                clock_account.clone()
            ],
            &instruction_data
        ),
        Err(ProgramError::InvalidArgument)
    );

    assert!(upd_price_no_fail_on_error(
        &program_id,
        &[
            funding_account.clone(),
            price_account.clone(),
            clock_account.clone()
        ],
        &instruction_data
    )
    .is_ok());

    {
        let price_data = load_checked::<pc_price_t>(&price_account, PC_VERSION).unwrap();
        assert_eq!(price_data.comp_[0].latest_.price_, 42);
        assert_eq!(price_data.comp_[0].latest_.conf_, 9);
        assert_eq!(price_data.comp_[0].latest_.pub_slot_, 1);
        assert_eq!(price_data.comp_[0].latest_.status_, PC_STATUS_UNKNOWN);
        assert_eq!(price_data.valid_slot_, 0);
        assert_eq!(price_data.agg_.pub_slot_, 1);
        assert_eq!(price_data.agg_.price_, 0);
        assert_eq!(price_data.agg_.status_, PC_STATUS_UNKNOWN);
    }
}


// Create an upd_price_no_fail_on_error instruction with the provided parameters
fn populate_instruction(instruction_data: &mut [u8], price: i64, conf: u64, pub_slot: u64) -> () {
    let mut cmd = load_mut::<cmd_upd_price_t>(instruction_data).unwrap();
    cmd.ver_ = PC_VERSION;
    cmd.cmd_ = command_t_e_cmd_upd_price_no_fail_on_error as i32;
    cmd.status_ = PC_STATUS_TRADING;
    cmd.price_ = price;
    cmd.conf_ = conf;
    cmd.pub_slot_ = pub_slot;
    cmd.unused_ = 0;
}
