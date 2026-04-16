// SPDX-License-Identifier: MPL-2.0
// Copyright ijl (2018-2026)

use crate::ffi::PyIntRef;
use crate::opt::{ALLOW_BIGINT, Opt, STRICT_INTEGER};
use crate::serialize::error::SerializeError;
use serde::ser::{Serialize, Serializer};

// https://tools.ietf.org/html/rfc7159#section-6
// "[-(2**53)+1, (2**53)-1]"
const STRICT_INT_MIN: i64 = -9007199254740991;
const STRICT_INT_MAX: i64 = 9007199254740991;

pub(crate) struct IntSerializer {
    ob: PyIntRef,
    opts: Opt,
}

impl IntSerializer {
    pub fn new(ob: PyIntRef, opts: Opt) -> Self {
        IntSerializer { ob: ob, opts: opts }
    }
}

impl Serialize for IntSerializer {
    #[inline(always)]
    #[cfg(feature = "inline_int")]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        unsafe {
            match self.ob.kind() {
                crate::ffi::PyIntKind::I32 => serializer.serialize_i32(self.ob.as_i32()),
                crate::ffi::PyIntKind::U32 => serializer.serialize_u32(self.ob.as_u32()),
                crate::ffi::PyIntKind::I64 => match self.ob.as_i64() {
                    Ok(value) => {
                        if opt_enabled!(self.opts, STRICT_INTEGER)
                            && !(STRICT_INT_MIN..=STRICT_INT_MAX).contains(&value)
                        {
                            cold_path!();
                            err!(SerializeError::Integer53Bits);
                        }
                        serializer.serialize_i64(value)
                    }
                    Err(_) => {
                        cold_path!();
                        if !opt_enabled!(self.opts, ALLOW_BIGINT) {
                            err!(SerializeError::Integer64Bits);
                        }
                        if opt_enabled!(self.opts, STRICT_INTEGER) {
                            err!(SerializeError::Integer53Bits);
                        }
                        let value = self
                            .ob
                            .as_i128()
                            .map_err(|_| {
                                serde::ser::Error::custom(SerializeError::Integer128Bits)
                            })?;
                        serializer.serialize_i128(value)
                    }
                },
                crate::ffi::PyIntKind::U64 => match self.ob.as_u64() {
                    Ok(value) => {
                        if opt_enabled!(self.opts, STRICT_INTEGER) && value > STRICT_INT_MAX as u64
                        {
                            cold_path!();
                            err!(SerializeError::Integer53Bits);
                        }
                        serializer.serialize_u64(value)
                    }
                    Err(_) => {
                        cold_path!();
                        if !opt_enabled!(self.opts, ALLOW_BIGINT) {
                            err!(SerializeError::Integer64Bits);
                        }
                        if opt_enabled!(self.opts, STRICT_INTEGER) {
                            err!(SerializeError::Integer53Bits);
                        }
                        let value = self
                            .ob
                            .as_u128()
                            .map_err(|_| {
                                serde::ser::Error::custom(SerializeError::Integer128Bits)
                            })?;
                        serializer.serialize_u128(value)
                    }
                },
                _ => {
                    cold_path!();
                    err!(SerializeError::Integer64Bits);
                }
            }
        }
    }

    #[inline(always)]
    #[cfg(not(feature = "inline_int"))]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        unsafe {
            match self.ob.as_i64() {
                Ok(value) => {
                    if opt_enabled!(self.opts, STRICT_INTEGER)
                        && !(STRICT_INT_MIN..=STRICT_INT_MAX).contains(&value)
                    {
                        cold_path!();
                        err!(SerializeError::Integer53Bits);
                    }
                    serializer.serialize_i64(value)
                }
                Err(_) => match self.ob.as_u64() {
                    Ok(value) => {
                        if opt_enabled!(self.opts, STRICT_INTEGER) && value > STRICT_INT_MAX as u64
                        {
                            cold_path!();
                            err!(SerializeError::Integer53Bits);
                        }
                        serializer.serialize_u64(value)
                    }
                    Err(_) => {
                        cold_path!();
                        if !opt_enabled!(self.opts, ALLOW_BIGINT) {
                            err!(SerializeError::Integer64Bits);
                        }
                        if opt_enabled!(self.opts, STRICT_INTEGER) {
                            err!(SerializeError::Integer53Bits);
                        }
                        match self.ob.as_i128() {
                            Ok(value) => serializer.serialize_i128(value),
                            Err(_) => match self.ob.as_u128() {
                                Ok(value) => serializer.serialize_u128(value),
                                Err(_) => err!(SerializeError::Integer128Bits),
                            },
                        }
                    }
                },
            }
        }
    }
}
