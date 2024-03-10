/*
 * Copyright (C) 2024 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! flag value module defines the flag value file format and methods for serialization
//! and deserialization

use crate::AconfigStorageError;
use crate::{read_str_from_bytes, read_u32_from_bytes, read_u8_from_bytes};
use std::fmt;

/// Flag value header struct
#[derive(PartialEq)]
pub struct FlagValueHeader {
    pub version: u32,
    pub container: String,
    pub file_size: u32,
    pub num_flags: u32,
    pub boolean_value_offset: u32,
}

/// Implement debug print trait for header
impl fmt::Debug for FlagValueHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "Version: {}, Container: {}, File Size: {}",
            self.version, self.container, self.file_size
        )?;
        writeln!(
            f,
            "Num of Flags: {}, Value Offset:{}",
            self.num_flags, self.boolean_value_offset
        )?;
        Ok(())
    }
}

impl FlagValueHeader {
    /// Serialize to bytes
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend_from_slice(&self.version.to_le_bytes());
        let container_bytes = self.container.as_bytes();
        result.extend_from_slice(&(container_bytes.len() as u32).to_le_bytes());
        result.extend_from_slice(container_bytes);
        result.extend_from_slice(&self.file_size.to_le_bytes());
        result.extend_from_slice(&self.num_flags.to_le_bytes());
        result.extend_from_slice(&self.boolean_value_offset.to_le_bytes());
        result
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, AconfigStorageError> {
        let mut head = 0;
        Ok(Self {
            version: read_u32_from_bytes(bytes, &mut head)?,
            container: read_str_from_bytes(bytes, &mut head)?,
            file_size: read_u32_from_bytes(bytes, &mut head)?,
            num_flags: read_u32_from_bytes(bytes, &mut head)?,
            boolean_value_offset: read_u32_from_bytes(bytes, &mut head)?,
        })
    }
}

/// Flag value list struct
#[derive(PartialEq)]
pub struct FlagValueList {
    pub header: FlagValueHeader,
    pub booleans: Vec<bool>,
}

/// Implement debug print trait for flag value
impl fmt::Debug for FlagValueList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Header:")?;
        write!(f, "{:?}", self.header)?;
        writeln!(f, "Values:")?;
        writeln!(f, "{:?}", self.booleans)?;
        Ok(())
    }
}

impl FlagValueList {
    /// Serialize to bytes
    pub fn as_bytes(&self) -> Vec<u8> {
        [
            self.header.as_bytes(),
            self.booleans.iter().map(|&v| u8::from(v).to_le_bytes()).collect::<Vec<_>>().concat(),
        ]
        .concat()
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, AconfigStorageError> {
        let header = FlagValueHeader::from_bytes(bytes)?;
        let num_flags = header.num_flags;
        let mut head = header.as_bytes().len();
        let booleans =
            (0..num_flags).map(|_| read_u8_from_bytes(bytes, &mut head).unwrap() == 1).collect();
        let list = Self { header, booleans };
        Ok(list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::create_test_flag_value_list;

    #[test]
    // this test point locks down the value list serialization
    fn test_serialization() {
        let flag_value_list = create_test_flag_value_list();

        let header: &FlagValueHeader = &flag_value_list.header;
        let reinterpreted_header = FlagValueHeader::from_bytes(&header.as_bytes());
        assert!(reinterpreted_header.is_ok());
        assert_eq!(header, &reinterpreted_header.unwrap());

        let reinterpreted_value_list = FlagValueList::from_bytes(&flag_value_list.as_bytes());
        assert!(reinterpreted_value_list.is_ok());
        assert_eq!(&flag_value_list, &reinterpreted_value_list.unwrap());
    }

    #[test]
    // this test point locks down that version number should be at the top of serialized
    // bytes
    fn test_version_number() {
        let flag_value_list = create_test_flag_value_list();
        let bytes = &flag_value_list.as_bytes();
        let mut head = 0;
        let version = read_u32_from_bytes(bytes, &mut head).unwrap();
        assert_eq!(version, 1234)
    }
}