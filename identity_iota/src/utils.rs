use core::iter::once;
use iota::{
    crypto::ternary::{
        sponge::{CurlP81, Sponge as _},
        Hash,
    },
    ternary::{raw::RawEncoding, Btrit, T1B1Buf, TritBuf, Trits, TryteBuf},
    transaction::bundled::{Address, BundledTransaction, BundledTransactionField as _},
};
use iota_conversion::trytes_converter;

use crate::error::{Error, Result};

pub fn txn_hash(txn: &BundledTransaction) -> Hash {
    let mut curl: CurlP81 = CurlP81::new();
    let mut tbuf: TritBuf<T1B1Buf> = TritBuf::zeros(BundledTransaction::trit_len());

    txn.into_trits_allocated(&mut tbuf);

    Hash::from_inner_unchecked(curl.digest(&tbuf).expect("infallible"))
}

pub fn txn_hash_trytes(txn: &BundledTransaction) -> String {
    encode_trits(txn_hash(txn).as_trits())
}

pub fn encode_trits<T: ?Sized>(trits: &Trits<T>) -> String
where
    T: RawEncoding<Trit = Btrit>,
{
    trits.iter_trytes().map(char::from).collect()
}

pub fn create_address_from_trits(trits: impl AsRef<str>) -> Result<Address> {
    let trits: TritBuf<T1B1Buf> = TryteBuf::try_from_str(trits.as_ref())?.as_trits().encode();

    Ok(Address::from_inner_unchecked(trits))
}

pub fn to_tryte(byte: u8) -> impl IntoIterator<Item = char> {
    once(iota_constants::TRYTE_ALPHABET[(byte % 27) as usize])
        .chain(once(iota_constants::TRYTE_ALPHABET[(byte / 27) as usize]))
}

pub fn utf8_to_trytes(input: impl AsRef<[u8]>) -> String {
    input.as_ref().iter().copied().flat_map(to_tryte).collect()
}

pub fn trytes_to_utf8(string: impl AsRef<str>) -> Result<String> {
    trytes_converter::to_string(string.as_ref()).map_err(|_| Error::InvalidTryteConversion)
}
