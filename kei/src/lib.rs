extern crate rand;
extern crate crc;

use rand::ChaChaRng;
use rand::SeedableRng;
use rand::Rng;
use std::{char, u32, u64};
use std::convert::{Into, AsRef};
use crc::crc64;

const A: [u32; 16] = [24, 53, 52, 66,
                      4, 3, 52, 466,
                      23, 92, 512, 625,
                      245, 562, 555, 667];
const B: [u32; 16] = [4, 2, 3, 6,
                      64, 34, 54, 64,
                      12, 34, 74, 36,
                      988, 467, 34, 23];
const C: [u32; 16] = [652425345, 328232592, 290284532, 344982339,
                      652545345, 328235392, 290225432, 301322339,
                      652415445, 321112592, 294284532, 304932339,
                      651425345, 325432592, 290684532, 304782339];

const BLACKLIST: [(u64, u64); 2] = [
    (0xffffffffffffffff, 0xffffffffffffffff),
    (0, 0),
];

#[inline(always)]
fn gen_key_value(seed: (u64, u64), a: u32, b: u32, c: u32) -> u32 {
    let seed_array = vec![(seed.0 >> 32) as u32, seed.0 as u32, (seed.1 >> 32) as u32, seed.1 as u32];
    let mut rng = ChaChaRng::from_seed(&seed_array);
    rng.set_counter(seed.1, seed.0);

    let a = a % 20;
    let b = b % 8;

    let tp: u32 = rng.gen();
    if a % 2 == 0 {
        tp >> a ^ ((tp >> b) | c)
    } else {
        tp >> a ^ ((tp >> b) & c)
    }
}

// From stack exchange
fn string64_generate(mut s: u64) -> String {
    let mut digits = vec![];
    while s > 0 {
        digits.push((s % 36) as u32);
        s /= 36;
    }
    let f = digits.into_iter().map(|x| char::from_digit(x, 36).unwrap()).rev().collect::<String>();
    if f.len() == 0 {
        "0".into()
    } else {
        f
    }
}

fn string32_generate(mut s: u32) -> String {
    let mut digits = vec![];
    while s > 0 {
        digits.push((s % 36) as u32);
        s /= 36;
    }
    let f = digits.into_iter().map(|x| char::from_digit(x, 36).unwrap()).rev().collect::<String>();
    if f.len() == 0 {
        "0".into()
    } else {
        f
    }
}

// We are going to represent it completely logically here, then implement an Into<String>
#[derive(Clone, Copy, Debug)]
pub struct Key {
    seed: (u64, u64),
    values: [u32; 16],
    userdata: [u64; 4],
}

impl Into<String> for Key {
    fn into(self) -> String {
        let mut format_str = format!("{}-{}", string64_generate(self.seed.0), string64_generate(self.seed.1));
        for i in 0..4 {
            format_str += &format!("-{}", string64_generate(self.userdata[i]));
        }
        for i in 0..16 {
            format_str += &format!("-{}", string32_generate(self.values[i]));
        }
        format_str
    }
}

impl Key {
    // Checksum against this rather than Into<String>
    pub fn secure_string(self) -> String {
        let mut format_str = format!("{}{}", string64_generate(self.seed.0), string64_generate(self.seed.1));
        for i in 0..4 {
            format_str += &format!("{}", string64_generate(self.userdata[i]));
        }
        for i in 0..16 {
            format_str += &format!("{}", string32_generate(self.values[i]));
        }
        format_str
    }

    pub fn parse_key<S: AsRef<str>>(s: S) -> Option<Key> {
        if let Some(index) = s.as_ref().rfind('-') {
            let (t, _) = s.as_ref().split_at(index);
            Key::_parse_key(t)
        } else {
            None
        }
    }

    /// Parses from "fancy" form into Key
    /// NOTE: Does not work from "secure_string" output
    fn _parse_key<S: AsRef<str>>(inp: S) -> Option<Key> {
        let mate = inp.as_ref().split('-');
        let seeds = mate.clone().take(2).map(|x| u64::from_str_radix(&x, 36).unwrap()).collect::<Vec<_>>();
        let userdata = mate.clone().skip(2).take(4).map(|x| u64::from_str_radix(&x, 36).unwrap()).collect::<Vec<_>>();
        let values = mate.clone().skip(6).map(|x| u32::from_str_radix(&x, 36).unwrap()).collect::<Vec<_>>();
        if values.len() > 16 { return None }
        let mut _values = [0; 16];
        for (i, v) in values.into_iter().enumerate() { _values[i] = v; }
        let mut _userdata = [0; 4];
        for (i, v) in userdata.into_iter().enumerate() { _userdata[i] = v; }
        Some(Key {
            seed: (seeds[0], seeds[1]),
            values: _values,
            userdata: _userdata
        })
    }

    pub fn generate(seed: (u64, u64)) -> Key {
        let mut key = Key {
            seed: seed,
            values: [0; 16],
            userdata: [0; 4]
        };
        // Remember: gen_key_value's functions MUST MATCH at both ends
        for (i, (a, b, c)) in A.into_iter().zip(B.into_iter()).zip(C.into_iter()).map(|((a, b), c)| (a, b, c)).enumerate() {
            key.values[i] = gen_key_value(key.seed, *a, *b, *c)
        }
        key
    }

    pub fn set_userdata(&mut self, ind: usize, v: u64) {
        if ind < 2 {
            // Hashed data (store hash)
            let hashy = vec![
                (v >> 52) as u8,
                (v >> 44) as u8,
                (v >> 36) as u8,
                (v >> 32) as u8,
                (v >> 24) as u8,
                (v >> 16) as u8,
                (v >> 8) as u8,
                (v) as u8
            ];
            self.userdata[ind] = crc64::checksum_iso(&hashy)
        } else {
            self.userdata[ind] = v
        }
    }

    pub fn userdata(&mut self, ind: usize) -> u64 {
        self.userdata[ind]
    }

    #[inline(always)]
    pub fn checksum(&self) -> String {
        let s = self.secure_string();
        let mut left: u32 = 0xdeadbeef;
        let mut right: u32 = 0x32323232;

        for (i, c) in s.bytes().enumerate() {
            let c: u32 = (c as u32).overflowing_shl(((i as u32)%4u32)*32u32).0;
            right = right.overflowing_add(c).0;
            left = left.overflowing_add(right).0;
        }

        string64_generate(((left as u64) << 32) + right as u64)
    }

    #[inline(always)]
    pub fn check_key(&self) -> KeyValidity {
        if BLACKLIST.contains(&self.seed) {
            KeyValidity::Blacklist
        } else {
            macro_rules! cheque {
                (chk $self_:expr => $a:ident as $i:expr) => (if $a != $self_.values[$i] {
                    return KeyValidity::Faux;
                });
                (gen $self_:expr => $a:ident as $i:expr) => (let $a = gen_key_value($self_.seed, A[$i], B[$i], C[$i]););
                (1) => {{
                    // 1
                    cheque!(gen self => k1 as 0);
                    cheque!(chk self => k1 as 0);
                }};
                (2) => {{
                    // 2
                    cheque!(gen self => k2 as 1);
                    cheque!(chk self => k2 as 1);
                }};
                (3) => {{
                    // 3
                    cheque!(gen self => k3 as 2);
                    cheque!(chk self => k3 as 2);
                }};
                (4) => {{
                    // 4
                    cheque!(gen self => k4 as 3);
                    cheque!(chk self => k4 as 3);
                }};
                (5) => {{
                    // 5
                    cheque!(gen self => k5 as 4);
                    cheque!(chk self => k5 as 4);
                }};
                (6) => {{
                    // 6
                    cheque!(gen self => k6 as 5);
                    cheque!(chk self => k6 as 5);
                }};
                (7) => {{
                    // 7
                    cheque!(gen self => k7 as 6);
                    cheque!(chk self => k7 as 6);
                }};
                (8) => {{
                    // 8
                    cheque!(gen self => k8 as 7);
                    cheque!(chk self => k8 as 7);
                }};
                (9) => {{
                    // 9
                    cheque!(gen self => k9 as 8);
                    cheque!(chk self => k9 as 8);
                }};
                (10) => {{
                    // 10
                    cheque!(gen self => k10 as 9);
                    cheque!(chk self => k10 as 9);
                }};
                (11) => {{
                    // 11
                    cheque!(gen self => k11 as 10);
                    cheque!(chk self => k11 as 10);
                }};
                (12) => {{
                    // 12
                    cheque!(gen self => k12 as 11);
                    cheque!(chk self => k12 as 11);
                }};
                (13) => {{
                    // 13
                    cheque!(gen self => k13 as 12);
                    cheque!(chk self => k13 as 12);
                }};
                (14) => {{
                    // 14
                    cheque!(gen self => k14 as 13);
                    cheque!(chk self => k14 as 13);
                }};
                (15) => {{
                    // 15
                    cheque!(gen self => k15 as 14);
                    cheque!(chk self => k15 as 14);
                }};
                (16) => {{
                    // 16
                    cheque!(gen self => k16 as 15);
                    cheque!(chk self => k16 as 15);
                }};
            };
            cheque!(6); cheque!(10);
            KeyValidity::Valid
        }
    }

    #[inline(always)]
    pub fn check_key_from_string<S: AsRef<str>>(s: S) -> KeyValidity {
        let s = s.as_ref().trim();
        if let Some(index) = s.rfind('-') {
            let (t, _) = s.split_at(index);
            if !check_checksum(s) { return KeyValidity::Invalid }
            match Key::parse_key(t) {
                Some(k) => k.check_key(),
                None => KeyValidity::Faux // Weird case where checksum passes, but the key doesn't parse
            }
        } else {
            KeyValidity::Invalid
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyValidity {
    Valid, // A correct key
    Invalid, // A key that fails checksum (misspelt)
    Blacklist, // A key that is banned
    Faux, // A key that a cracker or keygen has generated/used
}

#[inline(always)]
fn check_checksum<S: AsRef<str>>(s: S) -> bool {
    if let Some(index) = s.as_ref().rfind('-') {
        let (t, chk) = s.as_ref().split_at(index);
        let t = match Key::_parse_key(t) {
            Some(k) => k,
            None => return false
        };
        chk.trim_matches('-') == t.checksum()
    } else {
        false
    }
}
#[cfg(test)]
mod tests {
    use super::{Key, KeyValidity, gen_checksum};
    use rand::Rng;
    use rand;
    #[test]
    fn key_generation_validity() {
        let seed: (u64, u64) = rand::thread_rng().gen();
        let (key, chk) = Key::generate(seed);
        let key_text = format!("{}-{}", Into::<String>::into(key), chk);
        println!("{}", key_text);
        assert_eq!(Key::check_key_from_string(key_text), KeyValidity::Valid);
    }

    #[test]
    fn make_faux_key_test() {
        let seed: (u64, u64) = rand::thread_rng().gen();
        let key = Key {
            seed: seed,
            values: [134001; 16]
        };
        let chk = gen_checksum(key.secure_string());
        let key_text = format!("{}-{}", Into::<String>::into(key), chk);
        println!("{}", key_text);
        assert_eq!(Key::check_key_from_string(key_text), KeyValidity::Faux);
    }

    #[test]
    fn make_invalid_key_test() {
        let seed: (u64, u64) = rand::thread_rng().gen();
        let key = Key {
            seed: seed,
            values: [134001; 16]
        };
        let key_text = format!("{}-8u28ix", Into::<String>::into(key));
        println!("{}", key_text);
        assert_eq!(Key::check_key_from_string(key_text), KeyValidity::Invalid);
    }

    #[test]
    fn test_issued_keys() {
    }

    #[test]
    fn test_blacklist_keys() {
    }
}
