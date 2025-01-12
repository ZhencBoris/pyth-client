#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
//we do not use all the variables in oracle.h, so this helps with the warnings
#![allow(dead_code)]
//All the custom trait imports should go here
use bytemuck::{
    Pod,
    Zeroable,
};
use solana_program::pubkey::Pubkey;
use std::mem::size_of;
//bindings.rs is generated by build.rs to include
//things defined in bindings.h
include!("../bindings.rs");

/// The PythAccount trait's purpose is to attach constants to the 3 types of accounts that Pyth has
/// (mapping, price, product). This allows less duplicated code, because now we can create generic
/// functions to perform common checks on the accounts and to load and initialize the accounts.
pub trait PythAccount: Pod {
    /// `ACCOUNT_TYPE` is just the account discriminator, it is different for mapping, product and
    /// price
    const ACCOUNT_TYPE: u32;
    /// `INITIAL_SIZE` is the value that the field `size_` will take when the account is first
    /// initialized this one is slightly tricky because for mapping (resp. price) `size_` won't
    /// include the unpopulated entries of `prod_` (resp. `comp_`). At the beginning there are 0
    /// products (resp. 0 components) therefore `INITIAL_SIZE` will be equal to the offset of
    /// `prod_` (resp. `comp_`)  Similarly the product account `INITIAL_SIZE` won't include any
    /// key values.
    const INITIAL_SIZE: u32;
    /// `minimum_size()` is the minimum size that the solana account holding the struct needs to
    /// have. `INITIAL_SIZE` <= `minimum_size()`
    fn minimum_size() -> usize {
        size_of::<Self>()
    }
}

impl PythAccount for pc_map_table_t {
    const ACCOUNT_TYPE: u32 = PC_ACCTYPE_MAPPING;
    const INITIAL_SIZE: u32 = PC_MAP_TABLE_T_PROD_OFFSET as u32;
}

impl PythAccount for pc_prod_t {
    const ACCOUNT_TYPE: u32 = PC_ACCTYPE_PRODUCT;
    const INITIAL_SIZE: u32 = size_of::<pc_prod_t>() as u32;
    fn minimum_size() -> usize {
        PC_PROD_ACC_SIZE as usize
    }
}

impl PythAccount for pc_price_t {
    const ACCOUNT_TYPE: u32 = PC_ACCTYPE_PRICE;
    const INITIAL_SIZE: u32 = PC_PRICE_T_COMP_OFFSET as u32;
}

#[cfg(target_endian = "little")]
unsafe impl Zeroable for pc_acc {
}

#[cfg(target_endian = "little")]
unsafe impl Pod for pc_acc {
}

#[cfg(target_endian = "little")]
unsafe impl Zeroable for pc_map_table {
}

#[cfg(target_endian = "little")]
unsafe impl Pod for pc_map_table {
}

#[cfg(target_endian = "little")]
unsafe impl Zeroable for pc_prod {
}

#[cfg(target_endian = "little")]
unsafe impl Pod for pc_prod {
}

#[cfg(target_endian = "little")]
unsafe impl Zeroable for pc_price {
}

#[cfg(target_endian = "little")]
unsafe impl Pod for pc_price {
}

#[cfg(target_endian = "little")]
unsafe impl Zeroable for cmd_hdr {
}

#[cfg(target_endian = "little")]
unsafe impl Pod for cmd_hdr {
}

#[cfg(target_endian = "little")]
unsafe impl Zeroable for pc_price_info {
}

#[cfg(target_endian = "little")]
unsafe impl Pod for pc_price_info {
}

#[cfg(target_endian = "little")]
unsafe impl Zeroable for cmd_upd_price {
}

#[cfg(target_endian = "little")]
unsafe impl Pod for cmd_upd_price {
}

#[cfg(target_endian = "little")]
unsafe impl Zeroable for pc_ema {
}

#[cfg(target_endian = "little")]
unsafe impl Pod for pc_ema {
}

#[cfg(target_endian = "little")]
unsafe impl Zeroable for cmd_add_price_t {
}

#[cfg(target_endian = "little")]
unsafe impl Pod for cmd_add_price_t {
}

#[cfg(target_endian = "little")]
unsafe impl Zeroable for cmd_init_price_t {
}

#[cfg(target_endian = "little")]
unsafe impl Pod for cmd_init_price_t {
}

#[cfg(target_endian = "little")]
unsafe impl Zeroable for cmd_add_publisher_t {
}

#[cfg(target_endian = "little")]
unsafe impl Pod for cmd_add_publisher_t {
}

#[cfg(target_endian = "little")]
unsafe impl Zeroable for cmd_del_publisher_t {
}

#[cfg(target_endian = "little")]
unsafe impl Pod for cmd_del_publisher_t {
}

unsafe impl Zeroable for cmd_set_min_pub_t {
}

#[cfg(target_endian = "little")]
unsafe impl Pod for cmd_set_min_pub_t {
}

#[cfg(target_endian = "little")]
unsafe impl Zeroable for pc_pub_key_t {
}

#[cfg(target_endian = "little")]
unsafe impl Pod for pc_pub_key_t {
}


#[cfg(target_endian = "little")]
unsafe impl Zeroable for pc_price_comp_t {
}

#[cfg(target_endian = "little")]
unsafe impl Pod for pc_price_comp_t {
}

impl pc_pub_key_t {
    pub fn new_unique() -> pc_pub_key_t {
        let solana_unique = Pubkey::new_unique();
        pc_pub_key_t {
            k1_: solana_unique.to_bytes(),
        }
    }
}
