use base64::{Engine, engine::general_purpose::STANDARD};
use rayon::prelude::*;
use std::fmt;

/// Possible errors for AES operations.
#[derive(Debug)]
pub enum AESError {
    InvalidBase64(base64::DecodeError),
    InvalidUTF8(std::string::FromUtf8Error),
    InvalidKeyLength,
}

impl fmt::Display for AESError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AESError::InvalidBase64(err) => write!(f, "Base64 decoding error: {}", err),
            AESError::InvalidUTF8(err) => write!(f, "UTF-8 decoding error: {}", err),
            AESError::InvalidKeyLength => write!(f, "Invalid key length"),
        }
    }
}

impl std::error::Error for AESError {}

impl From<base64::DecodeError> for AESError {
    fn from(err: base64::DecodeError) -> Self {
        AESError::InvalidBase64(err)
    }
}

impl From<std::string::FromUtf8Error> for AESError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        AESError::InvalidUTF8(err)
    }
}

/// Lookup Tables and Lookup Functions
static TABLE: [[u8; 256]; 6]  = [
    [0x00,0x02,0x04,0x06,0x08,0x0a,0x0c,0x0e,0x10,0x12,0x14,0x16,0x18,0x1a,0x1c,0x1e,
     0x20,0x22,0x24,0x26,0x28,0x2a,0x2c,0x2e,0x30,0x32,0x34,0x36,0x38,0x3a,0x3c,0x3e,
     0x40,0x42,0x44,0x46,0x48,0x4a,0x4c,0x4e,0x50,0x52,0x54,0x56,0x58,0x5a,0x5c,0x5e,
     0x60,0x62,0x64,0x66,0x68,0x6a,0x6c,0x6e,0x70,0x72,0x74,0x76,0x78,0x7a,0x7c,0x7e,
     0x80,0x82,0x84,0x86,0x88,0x8a,0x8c,0x8e,0x90,0x92,0x94,0x96,0x98,0x9a,0x9c,0x9e,
     0xa0,0xa2,0xa4,0xa6,0xa8,0xaa,0xac,0xae,0xb0,0xb2,0xb4,0xb6,0xb8,0xba,0xbc,0xbe,
     0xc0,0xc2,0xc4,0xc6,0xc8,0xca,0xcc,0xce,0xd0,0xd2,0xd4,0xd6,0xd8,0xda,0xdc,0xde,
     0xe0,0xe2,0xe4,0xe6,0xe8,0xea,0xec,0xee,0xf0,0xf2,0xf4,0xf6,0xf8,0xfa,0xfc,0xfe,
     0x1b,0x19,0x1f,0x1d,0x13,0x11,0x17,0x15,0x0b,0x09,0x0f,0x0d,0x03,0x01,0x07,0x05,
     0x3b,0x39,0x3f,0x3d,0x33,0x31,0x37,0x35,0x2b,0x29,0x2f,0x2d,0x23,0x21,0x27,0x25,
     0x5b,0x59,0x5f,0x5d,0x53,0x51,0x57,0x55,0x4b,0x49,0x4f,0x4d,0x43,0x41,0x47,0x45,
     0x7b,0x79,0x7f,0x7d,0x73,0x71,0x77,0x75,0x6b,0x69,0x6f,0x6d,0x63,0x61,0x67,0x65,
     0x9b,0x99,0x9f,0x9d,0x93,0x91,0x97,0x95,0x8b,0x89,0x8f,0x8d,0x83,0x81,0x87,0x85,
     0xbb,0xb9,0xbf,0xbd,0xb3,0xb1,0xb7,0xb5,0xab,0xa9,0xaf,0xad,0xa3,0xa1,0xa7,0xa5,
     0xdb,0xd9,0xdf,0xdd,0xd3,0xd1,0xd7,0xd5,0xcb,0xc9,0xcf,0xcd,0xc3,0xc1,0xc7,0xc5,
     0xfb,0xf9,0xff,0xfd,0xf3,0xf1,0xf7,0xf5,0xeb,0xe9,0xef,0xed,0xe3,0xe1,0xe7,0xe5],
    [0x00,0x03,0x06,0x05,0x0c,0x0f,0x0a,0x09,0x18,0x1b,0x1e,0x1d,0x14,0x17,0x12,0x11,
        0x30,0x33,0x36,0x35,0x3c,0x3f,0x3a,0x39,0x28,0x2b,0x2e,0x2d,0x24,0x27,0x22,0x21,
        0x60,0x63,0x66,0x65,0x6c,0x6f,0x6a,0x69,0x78,0x7b,0x7e,0x7d,0x74,0x77,0x72,0x71,
        0x50,0x53,0x56,0x55,0x5c,0x5f,0x5a,0x59,0x48,0x4b,0x4e,0x4d,0x44,0x47,0x42,0x41,
        0xc0,0xc3,0xc6,0xc5,0xcc,0xcf,0xca,0xc9,0xd8,0xdb,0xde,0xdd,0xd4,0xd7,0xd2,0xd1,
        0xf0,0xf3,0xf6,0xf5,0xfc,0xff,0xfa,0xf9,0xe8,0xeb,0xee,0xed,0xe4,0xe7,0xe2,0xe1,
        0xa0,0xa3,0xa6,0xa5,0xac,0xaf,0xaa,0xa9,0xb8,0xbb,0xbe,0xbd,0xb4,0xb7,0xb2,0xb1,
        0x90,0x93,0x96,0x95,0x9c,0x9f,0x9a,0x99,0x88,0x8b,0x8e,0x8d,0x84,0x87,0x82,0x81,
        0x9b,0x98,0x9d,0x9e,0x97,0x94,0x91,0x92,0x83,0x80,0x85,0x86,0x8f,0x8c,0x89,0x8a,
        0xab,0xa8,0xad,0xae,0xa7,0xa4,0xa1,0xa2,0xb3,0xb0,0xb5,0xb6,0xbf,0xbc,0xb9,0xba,
        0xfb,0xf8,0xfd,0xfe,0xf7,0xf4,0xf1,0xf2,0xe3,0xe0,0xe5,0xe6,0xef,0xec,0xe9,0xea,
        0xcb,0xc8,0xcd,0xce,0xc7,0xc4,0xc1,0xc2,0xd3,0xd0,0xd5,0xd6,0xdf,0xdc,0xd9,0xda,
        0x5b,0x58,0x5d,0x5e,0x57,0x54,0x51,0x52,0x43,0x40,0x45,0x46,0x4f,0x4c,0x49,0x4a,
        0x6b,0x68,0x6d,0x6e,0x67,0x64,0x61,0x62,0x73,0x70,0x75,0x76,0x7f,0x7c,0x79,0x7a,
        0x3b,0x38,0x3d,0x3e,0x37,0x34,0x31,0x32,0x23,0x20,0x25,0x26,0x2f,0x2c,0x29,0x2a,
        0x0b,0x08,0x0d,0x0e,0x07,0x04,0x01,0x02,0x13,0x10,0x15,0x16,0x1f,0x1c,0x19,0x1a],
    [0x00,0x09,0x12,0x1b,0x24,0x2d,0x36,0x3f,0x48,0x41,0x5a,0x53,0x6c,0x65,0x7e,0x77,
        0x90,0x99,0x82,0x8b,0xb4,0xbd,0xa6,0xaf,0xd8,0xd1,0xca,0xc3,0xfc,0xf5,0xee,0xe7,
        0x3b,0x32,0x29,0x20,0x1f,0x16,0x0d,0x04,0x73,0x7a,0x61,0x68,0x57,0x5e,0x45,0x4c,
        0xab,0xa2,0xb9,0xb0,0x8f,0x86,0x9d,0x94,0xe3,0xea,0xf1,0xf8,0xc7,0xce,0xd5,0xdc,
        0x76,0x7f,0x64,0x6d,0x52,0x5b,0x40,0x49,0x3e,0x37,0x2c,0x25,0x1a,0x13,0x08,0x01,
        0xe6,0xef,0xf4,0xfd,0xc2,0xcb,0xd0,0xd9,0xae,0xa7,0xbc,0xb5,0x8a,0x83,0x98,0x91,
        0x4d,0x44,0x5f,0x56,0x69,0x60,0x7b,0x72,0x05,0x0c,0x17,0x1e,0x21,0x28,0x33,0x3a,
        0xdd,0xd4,0xcf,0xc6,0xf9,0xf0,0xeb,0xe2,0x95,0x9c,0x87,0x8e,0xb1,0xb8,0xa3,0xaa,
        0xec,0xe5,0xfe,0xf7,0xc8,0xc1,0xda,0xd3,0xa4,0xad,0xb6,0xbf,0x80,0x89,0x92,0x9b,
        0x7c,0x75,0x6e,0x67,0x58,0x51,0x4a,0x43,0x34,0x3d,0x26,0x2f,0x10,0x19,0x02,0x0b,
        0xd7,0xde,0xc5,0xcc,0xf3,0xfa,0xe1,0xe8,0x9f,0x96,0x8d,0x84,0xbb,0xb2,0xa9,0xa0,
        0x47,0x4e,0x55,0x5c,0x63,0x6a,0x71,0x78,0x0f,0x06,0x1d,0x14,0x2b,0x22,0x39,0x30,
        0x9a,0x93,0x88,0x81,0xbe,0xb7,0xac,0xa5,0xd2,0xdb,0xc0,0xc9,0xf6,0xff,0xe4,0xed,
        0x0a,0x03,0x18,0x11,0x2e,0x27,0x3c,0x35,0x42,0x4b,0x50,0x59,0x66,0x6f,0x74,0x7d,
        0xa1,0xa8,0xb3,0xba,0x85,0x8c,0x97,0x9e,0xe9,0xe0,0xfb,0xf2,0xcd,0xc4,0xdf,0xd6,
        0x31,0x38,0x23,0x2a,0x15,0x1c,0x07,0x0e,0x79,0x70,0x6b,0x62,0x5d,0x54,0x4f,0x46],
    [0x00,0x0b,0x16,0x1d,0x2c,0x27,0x3a,0x31,0x58,0x53,0x4e,0x45,0x74,0x7f,0x62,0x69,
        0xb0,0xbb,0xa6,0xad,0x9c,0x97,0x8a,0x81,0xe8,0xe3,0xfe,0xf5,0xc4,0xcf,0xd2,0xd9,
        0x7b,0x70,0x6d,0x66,0x57,0x5c,0x41,0x4a,0x23,0x28,0x35,0x3e,0x0f,0x04,0x19,0x12,
        0xcb,0xc0,0xdd,0xd6,0xe7,0xec,0xf1,0xfa,0x93,0x98,0x85,0x8e,0xbf,0xb4,0xa9,0xa2,
        0xf6,0xfd,0xe0,0xeb,0xda,0xd1,0xcc,0xc7,0xae,0xa5,0xb8,0xb3,0x82,0x89,0x94,0x9f,
        0x46,0x4d,0x50,0x5b,0x6a,0x61,0x7c,0x77,0x1e,0x15,0x08,0x03,0x32,0x39,0x24,0x2f,
        0x8d,0x86,0x9b,0x90,0xa1,0xaa,0xb7,0xbc,0xd5,0xde,0xc3,0xc8,0xf9,0xf2,0xef,0xe4,
        0x3d,0x36,0x2b,0x20,0x11,0x1a,0x07,0x0c,0x65,0x6e,0x73,0x78,0x49,0x42,0x5f,0x54,
        0xf7,0xfc,0xe1,0xea,0xdb,0xd0,0xcd,0xc6,0xaf,0xa4,0xb9,0xb2,0x83,0x88,0x95,0x9e,
        0x47,0x4c,0x51,0x5a,0x6b,0x60,0x7d,0x76,0x1f,0x14,0x09,0x02,0x33,0x38,0x25,0x2e,
        0x8c,0x87,0x9a,0x91,0xa0,0xab,0xb6,0xbd,0xd4,0xdf,0xc2,0xc9,0xf8,0xf3,0xee,0xe5,
        0x3c,0x37,0x2a,0x21,0x10,0x1b,0x06,0x0d,0x64,0x6f,0x72,0x79,0x48,0x43,0x5e,0x55,
        0x01,0x0a,0x17,0x1c,0x2d,0x26,0x3b,0x30,0x59,0x52,0x4f,0x44,0x75,0x7e,0x63,0x68,
        0xb1,0xba,0xa7,0xac,0x9d,0x96,0x8b,0x80,0xe9,0xe2,0xff,0xf4,0xc5,0xce,0xd3,0xd8,
        0x7a,0x71,0x6c,0x67,0x56,0x5d,0x40,0x4b,0x22,0x29,0x34,0x3f,0x0e,0x05,0x18,0x13,
        0xca,0xc1,0xdc,0xd7,0xe6,0xed,0xf0,0xfb,0x92,0x99,0x84,0x8f,0xbe,0xb5,0xa8,0xa3],
    [0x00,0x0d,0x1a,0x17,0x34,0x39,0x2e,0x23,0x68,0x65,0x72,0x7f,0x5c,0x51,0x46,0x4b,
        0xd0,0xdd,0xca,0xc7,0xe4,0xe9,0xfe,0xf3,0xb8,0xb5,0xa2,0xaf,0x8c,0x81,0x96,0x9b,
        0xbb,0xb6,0xa1,0xac,0x8f,0x82,0x95,0x98,0xd3,0xde,0xc9,0xc4,0xe7,0xea,0xfd,0xf0,
        0x6b,0x66,0x71,0x7c,0x5f,0x52,0x45,0x48,0x03,0x0e,0x19,0x14,0x37,0x3a,0x2d,0x20,
        0x6d,0x60,0x77,0x7a,0x59,0x54,0x43,0x4e,0x05,0x08,0x1f,0x12,0x31,0x3c,0x2b,0x26,
        0xbd,0xb0,0xa7,0xaa,0x89,0x84,0x93,0x9e,0xd5,0xd8,0xcf,0xc2,0xe1,0xec,0xfb,0xf6,
        0xd6,0xdb,0xcc,0xc1,0xe2,0xef,0xf8,0xf5,0xbe,0xb3,0xa4,0xa9,0x8a,0x87,0x90,0x9d,
        0x06,0x0b,0x1c,0x11,0x32,0x3f,0x28,0x25,0x6e,0x63,0x74,0x79,0x5a,0x57,0x40,0x4d,
        0xda,0xd7,0xc0,0xcd,0xee,0xe3,0xf4,0xf9,0xb2,0xbf,0xa8,0xa5,0x86,0x8b,0x9c,0x91,
        0x0a,0x07,0x10,0x1d,0x3e,0x33,0x24,0x29,0x62,0x6f,0x78,0x75,0x56,0x5b,0x4c,0x41,
        0x61,0x6c,0x7b,0x76,0x55,0x58,0x4f,0x42,0x09,0x04,0x13,0x1e,0x3d,0x30,0x27,0x2a,
        0xb1,0xbc,0xab,0xa6,0x85,0x88,0x9f,0x92,0xd9,0xd4,0xc3,0xce,0xed,0xe0,0xf7,0xfa,
        0xb7,0xba,0xad,0xa0,0x83,0x8e,0x99,0x94,0xdf,0xd2,0xc5,0xc8,0xeb,0xe6,0xf1,0xfc,
        0x67,0x6a,0x7d,0x70,0x53,0x5e,0x49,0x44,0x0f,0x02,0x15,0x18,0x3b,0x36,0x21,0x2c,
        0x0c,0x01,0x16,0x1b,0x38,0x35,0x22,0x2f,0x64,0x69,0x7e,0x73,0x50,0x5d,0x4a,0x47,
        0xdc,0xd1,0xc6,0xcb,0xe8,0xe5,0xf2,0xff,0xb4,0xb9,0xae,0xa3,0x80,0x8d,0x9a,0x97],
[0x00,0x0e,0x1c,0x12,0x38,0x36,0x24,0x2a,0x70,0x7e,0x6c,0x62,0x48,0x46,0x54,0x5a,
        0xe0,0xee,0xfc,0xf2,0xd8,0xd6,0xc4,0xca,0x90,0x9e,0x8c,0x82,0xa8,0xa6,0xb4,0xba,
        0xdb,0xd5,0xc7,0xc9,0xe3,0xed,0xff,0xf1,0xab,0xa5,0xb7,0xb9,0x93,0x9d,0x8f,0x81,
        0x3b,0x35,0x27,0x29,0x03,0x0d,0x1f,0x11,0x4b,0x45,0x57,0x59,0x73,0x7d,0x6f,0x61,
        0xad,0xa3,0xb1,0xbf,0x95,0x9b,0x89,0x87,0xdd,0xd3,0xc1,0xcf,0xe5,0xeb,0xf9,0xf7,
        0x4d,0x43,0x51,0x5f,0x75,0x7b,0x69,0x67,0x3d,0x33,0x21,0x2f,0x05,0x0b,0x19,0x17,
        0x76,0x78,0x6a,0x64,0x4e,0x40,0x52,0x5c,0x06,0x08,0x1a,0x14,0x3e,0x30,0x22,0x2c,
        0x96,0x98,0x8a,0x84,0xae,0xa0,0xb2,0xbc,0xe6,0xe8,0xfa,0xf4,0xde,0xd0,0xc2,0xcc,
        0x41,0x4f,0x5d,0x53,0x79,0x77,0x65,0x6b,0x31,0x3f,0x2d,0x23,0x09,0x07,0x15,0x1b,
        0xa1,0xaf,0xbd,0xb3,0x99,0x97,0x85,0x8b,0xd1,0xdf,0xcd,0xc3,0xe9,0xe7,0xf5,0xfb,
        0x9a,0x94,0x86,0x88,0xa2,0xac,0xbe,0xb0,0xea,0xe4,0xf6,0xf8,0xd2,0xdc,0xce,0xc0,
        0x7a,0x74,0x66,0x68,0x42,0x4c,0x5e,0x50,0x0a,0x04,0x16,0x18,0x32,0x3c,0x2e,0x20,
        0xec,0xe2,0xf0,0xfe,0xd4,0xda,0xc8,0xc6,0x9c,0x92,0x80,0x8e,0xa4,0xaa,0xb8,0xb6,
        0x0c,0x02,0x10,0x1e,0x34,0x3a,0x28,0x26,0x7c,0x72,0x60,0x6e,0x44,0x4a,0x58,0x56,
        0x37,0x39,0x2b,0x25,0x0f,0x01,0x13,0x1d,0x47,0x49,0x5b,0x55,0x7f,0x71,0x63,0x6d,
        0xd7,0xd9,0xcb,0xc5,0xef,0xe1,0xf3,0xfd,0xa7,0xa9,0xbb,0xb5,0x9f,0x91,0x83,0x8d]];

static AES_SBOX: [[u8;16];16] = [[99, 124, 119, 123, 242, 107, 111, 197, 48, 1, 103, 43, 254, 215, 171, 118],
    [202, 130, 201, 125, 250, 89, 71, 240, 173, 212, 162, 175, 156, 164, 114, 192],
    [183, 253, 147, 38, 54, 63, 247, 204, 52, 165, 229, 241, 113, 216, 49, 21],
    [4, 199, 35, 195, 24, 150, 5, 154, 7, 18, 128, 226, 235, 39, 178, 117],
    [9, 131, 44, 26, 27, 110, 90, 160, 82, 59, 214, 179, 41, 227, 47, 132],
    [83, 209, 0, 237, 32, 252, 177, 91, 106, 203, 190, 57, 74, 76, 88, 207],
    [208, 239, 170, 251, 67, 77, 51, 133, 69, 249, 2, 127, 80, 60, 159, 168],
    [81, 163, 64, 143, 146, 157, 56, 245, 188, 182, 218, 33, 16, 255, 243, 210],
    [205, 12, 19, 236, 95, 151, 68, 23, 196, 167, 126, 61, 100, 93, 25, 115],
    [96, 129, 79, 220, 34, 42, 144, 136, 70, 238, 184, 20, 222, 94, 11, 219],
    [224, 50, 58, 10, 73, 6, 36, 92, 194, 211, 172, 98, 145, 149, 228, 121],
[231, 200, 55, 109, 141, 213, 78, 169, 108, 86, 244, 234, 101, 122, 174, 8],
[186, 120, 37, 46, 28, 166, 180, 198, 232, 221, 116, 31, 75, 189, 139, 138],
[112, 62, 181, 102, 72, 3, 246, 14, 97, 53, 87, 185, 134, 193, 29, 158],
[225, 248, 152, 17, 105, 217, 142, 148, 155, 30, 135, 233, 206, 85, 40, 223],
[140, 161, 137, 13, 191, 230, 66, 104, 65, 153, 45, 15, 176, 84, 187, 22]];

static REVERSE_AES_SBOX: [[u8;16];16] = [[82, 9, 106, 213, 48, 54, 165, 56, 191, 64, 163, 158, 129, 243, 215, 251],
    [124, 227, 57, 130, 155, 47, 255, 135, 52, 142, 67, 68, 196, 222, 233, 203],
    [84, 123, 148, 50, 166, 194, 35, 61, 238, 76, 149, 11, 66, 250, 195, 78],
    [8, 46, 161, 102, 40, 217, 36, 178, 118, 91, 162, 73, 109, 139, 209, 37],
    [114, 248, 246, 100, 134, 104, 152, 22, 212, 164, 92, 204, 93, 101, 182, 146],
    [108, 112, 72, 80, 253, 237, 185, 218, 94, 21, 70, 87, 167, 141, 157, 132],
    [144, 216, 171, 0, 140, 188, 211, 10, 247, 228, 88, 5, 184, 179, 69, 6],
    [208, 44, 30, 143, 202, 63, 15, 2, 193, 175, 189, 3, 1, 19, 138, 107],
    [58, 145, 17, 65, 79, 103, 220, 234, 151, 242, 207, 206, 240, 180, 230, 115],
    [150, 172, 116, 34, 231, 173, 53, 133, 226, 249, 55, 232, 28, 117, 223, 110],
    [71, 241, 26, 113, 29, 41, 197, 137, 111, 183, 98, 14, 170, 24, 190, 27],
    [252, 86, 62, 75, 198, 210, 121, 32, 154, 219, 192, 254, 120, 205, 90, 244],
    [31, 221, 168, 51, 136, 7, 199, 49, 177, 18, 16, 89, 39, 128, 236, 95],
    [96, 81, 127, 169, 25, 181, 74, 13, 45, 229, 122, 159, 147, 201, 156, 239],
    [160, 224, 59, 77, 174, 42, 245, 176, 200, 235, 187, 60, 131, 83, 153, 97],
    [23, 43, 4, 126, 186, 119, 214, 38, 225, 105, 20, 99, 85, 33, 12, 125]];

static MIX_MATRIX:[[u8;4];4]= [
    [2,3,1,1],
    [1,2,3,1],
    [1,1,2,3],
    [3,1,1,2]];

static INV_MIX_MATRIX:[[u8;4];4]= [
    [0x0e,0x0b,0x0d,0x09],
    [0x09,0x0e,0x0b,0x0d],
    [0x0d,0x09,0x0e,0x0b],
    [0x0b,0x0d,0x09,0x0e]];

static RC: [u8;22] = [0x01,0x02,0x04,0x08,0x10,0x20,0x40,0x80,0x1B,0x36,0x6C,0xDB,0xAB,0x4D,0x9A,0x2F,0x5E,0xBC,0x63,0xC6,0x97,0x35];


fn lookup(byte: u8) -> u8 {
    let x = (byte >> 4) as usize;
    let y = (byte & 15) as usize;

    AES_SBOX.get(x)
        .and_then(|row| row.get(y))
        .copied()
        .unwrap_or_else(|| panic!("Invalid index in AES_SBOX for byte: {}", byte))
}


fn reverse_lookup(byte: u8) -> u8 {
    let x = (byte >> 4) as usize;
    let y = (byte & 15) as usize;

    REVERSE_AES_SBOX.get(x)
        .and_then(|row| row.get(y))
        .copied()
        .unwrap_or_else(|| panic!("Invalid index in REVERSE_AES_SBOX for byte: {}", byte))
}
fn round_constant(round:u8)->u8{
    RC.get(round as usize).copied().unwrap_or_else(|| panic!("Invalid round: {}", round))
}

fn table_index(n:u8)-> usize{
    match n {
        2 => 0,
        3 => 1,
        9 => 2,
        11 => 3,
        13 => 4,
        14 => 5,
        _ => {panic!("Invalid table index: {}", n)}
    }
}
fn gmul(n: u8, m: u8) -> u8 {
    match n {
        1 => m,
        _ => {
            let index = table_index(n);
            match TABLE.get(index) {
                Some(row) => {
                    if let Some(&value) = row.get(m as usize) {
                        value
                    } else {
                        panic!("Index m={} out of bounds in TABLE for n={}", m, n);
                    }
                },
                None => panic!("Value n={} not found in TABLE", n),
            }
        }
    }
}

#[derive(Debug)]
pub struct AESteve {
    keys: [[[u8; 4]; 4]; 11],
}


impl AESteve {
    /// Creates a new AES instance with the given 128-bit key.
    ///
    /// # Arguments
    ///
    /// * `key` - The 128-bit key (16 bytes).
    ///
    /// # Errors
    ///
    /// Returns `AESError::InvalidKeyLength` if the key length is not 16 bytes.
    pub fn new(key: &[u8]) -> Result<Self, AESError> {
        if key.len() != 16 {
            return Err(AESError::InvalidKeyLength);
        }
        let mut key_array = [0u8; 16];
        key_array.copy_from_slice(key);
        let keys = Self::expand_key(&key_array);
        Ok(AESteve { keys })
    }

    fn expand_key(key: &[u8; 16]) -> [[[u8; 4]; 4]; 11] {
        // Key expansion implementation
        let mut keys: [[[u8; 4]; 4]; 11] = [[[0; 4]; 4]; 11];
        for col_idx in 0..4 {
            keys[0][col_idx].copy_from_slice(&key[col_idx * 4..(col_idx + 1) * 4]);
        }

        for round_idx in 0..10 {
            for col_idx in 0..4 {
                if col_idx == 0 {
                    let last_col = keys[round_idx][3];
                    let rotated_col = [last_col[1], last_col[2], last_col[3], last_col[0]];
                    let t1 = keys[round_idx][0][0] ^ (lookup(rotated_col[0]) ^ round_constant(round_idx as u8));
                    let t2 = keys[round_idx][0][1] ^ lookup(rotated_col[1]);
                    let t3 = keys[round_idx][0][2] ^ lookup(rotated_col[2]);
                    let t4 = keys[round_idx][0][3] ^ lookup(rotated_col[3]);
                    keys[round_idx + 1][0] = [t1, t2, t3, t4];
                } else {
                    let t1 = keys[round_idx + 1][col_idx - 1][0] ^ keys[round_idx][col_idx][0];
                    let t2 = keys[round_idx + 1][col_idx - 1][1] ^ keys[round_idx][col_idx][1];
                    let t3 = keys[round_idx + 1][col_idx - 1][2] ^ keys[round_idx][col_idx][2];
                    let t4 = keys[round_idx + 1][col_idx - 1][3] ^ keys[round_idx][col_idx][3];
                    keys[round_idx + 1][col_idx] = [t1, t2, t3, t4];
                }
            }
        }
        keys
    }

    fn pad(mut message: Vec<u8>) -> Vec<u8> {
        message.push(0x80);
        while message.len() % 16 != 0 {
            message.push(0x00);
        }
        message
    }

    fn depad(message: Vec<u8>) -> Vec<u8> {
        if let Some(pos) = message.iter().position(|&n| n == 0x80) {
            message[0..pos].to_vec()
        } else {
            message
        }
    }

    fn make_blocks(padded_message: Vec<u8>) -> Vec<[[u8; 4]; 4]> {
        let mut blocks = Vec::new();
        for chunk in padded_message.chunks(16) {
            let mut block = [[0u8; 4]; 4];
            for (i, &byte) in chunk.iter().enumerate() {
                block[i / 4][i % 4] = byte;
            }
            blocks.push(block);
        }
        blocks
    }

    fn add_round_key(key: [[u8; 4]; 4], block: [[u8; 4]; 4]) -> [[u8; 4]; 4] {
        let mut new_block = [[0u8; 4]; 4];
        for col_idx in 0..4 {
            for row_idx in 0..4 {
                new_block[col_idx][row_idx] = key[col_idx][row_idx] ^ block[col_idx][row_idx];
            }
        }
        new_block
    }

    fn sub_bytes(block: [[u8; 4]; 4]) -> [[u8; 4]; 4] {
        let mut new_block = [[0u8; 4]; 4];
        for col_idx in 0..4 {
            for row_idx in 0..4 {
                new_block[col_idx][row_idx] = lookup(block[col_idx][row_idx]);
            }
        }
        new_block
    }

    fn inv_sub_bytes(block: [[u8; 4]; 4]) -> [[u8; 4]; 4] {
        let mut new_block = [[0u8; 4]; 4];
        for col_idx in 0..4 {
            for row_idx in 0..4 {
                new_block[col_idx][row_idx] = reverse_lookup(block[col_idx][row_idx]);
            }
        }
        new_block
    }

    fn shift_rows(block: [[u8; 4]; 4]) -> [[u8; 4]; 4] {
        let mut new_block = [[0u8; 4]; 4];
        for i in 0..4 {
            new_block[i][0] = block[i][0];
            new_block[i][1] = block[(i + 1) % 4][1];
            new_block[i][2] = block[(i + 2) % 4][2];
            new_block[i][3] = block[(i + 3) % 4][3];
        }
        new_block
    }

    fn inv_shift_rows(block: [[u8; 4]; 4]) -> [[u8; 4]; 4] {
        let mut new_block = [[0u8; 4]; 4];
        for i in 0..4 {
            new_block[i][0] = block[i][0];
            new_block[i][1] = block[(i + 3) % 4][1];
            new_block[i][2] = block[(i + 2) % 4][2];
            new_block[i][3] = block[(i + 1) % 4][3];
        }
        new_block
    }

    fn transpose(block: [[u8; 4]; 4]) -> [[u8; 4]; 4] {
        let mut new_block = [[0u8; 4]; 4];
        for col_idx in 0..4 {
            for row_idx in 0..4 {
                new_block[col_idx][row_idx] = block[row_idx][col_idx];
            }
        }
        new_block
    }

    fn mix_columns(block: [[u8; 4]; 4]) -> [[u8; 4]; 4] {
        let block_t = Self::transpose(block);
        let mut new_block = [[0u8; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    new_block[j][i] ^= gmul(MIX_MATRIX[i][k], block_t[k][j]);
                }
            }
        }
        new_block
    }

    fn inv_mix_columns(block: [[u8; 4]; 4]) -> [[u8; 4]; 4] {
        let block_t = Self::transpose(block);
        let mut new_block = [[0u8; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    new_block[j][i] ^= gmul(INV_MIX_MATRIX[i][k], block_t[k][j]);
                }
            }
        }
        new_block
    }

    fn encrypt_block(&self, block: [[u8; 4]; 4]) -> [[u8; 4]; 4] {
        let mut new_block = Self::add_round_key(self.keys[0], block);
        for i in 0..9 {
            new_block = Self::sub_bytes(new_block);
            new_block = Self::shift_rows(new_block);
            new_block = Self::mix_columns(new_block);
            new_block = Self::add_round_key(self.keys[i + 1], new_block);
        }
        new_block = Self::sub_bytes(new_block);
        new_block = Self::shift_rows(new_block);
        new_block = Self::add_round_key(self.keys[10], new_block);

        new_block
    }

    fn decrypt_block(&self, block: [[u8; 4]; 4]) -> [[u8; 4]; 4] {
        let mut new_block = Self::add_round_key(self.keys[10], block);
        new_block = Self::inv_shift_rows(new_block);
        new_block = Self::inv_sub_bytes(new_block);
        for i in 0..9 {
            new_block = Self::add_round_key(self.keys[9 - i], new_block);
            new_block = Self::inv_mix_columns(new_block);
            new_block = Self::inv_shift_rows(new_block);
            new_block = Self::inv_sub_bytes(new_block);
        }
        new_block = Self::add_round_key(self.keys[0], new_block);

        new_block
    }

    /// Encrypts the given message.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to be encrypted.
    ///
    /// # Returns
    ///
    /// * `String` - The encrypted message in Base64 format.
    ///
    /// # Errors
    ///
    /// Returns `AESError` if an error occurs during encryption.
    pub fn encrypt(&self, message: String) -> Result<String, AESError> {
        let message = message.as_bytes().to_vec();
        let padded_message = Self::pad(message);
        let blocks = Self::make_blocks(padded_message);

        let encrypted_blocks: Vec<[[u8; 4]; 4]> = blocks
            .into_par_iter()
            .map(|block| self.encrypt_block(block))
            .collect();

        let flattened: Vec<u8> = encrypted_blocks
            .into_iter()
            .flat_map(|array4x4| array4x4.into_iter().flat_map(|array4| array4.into_iter()))
            .collect();
        Ok(STANDARD.encode(&flattened))
    }

    /// Decrypts the given encrypted message.
    ///
    /// # Arguments
    ///
    /// * `encrypted_message` - The encrypted message in Base64 format.
    ///
    /// # Returns
    ///
    /// * `String` - The decrypted message.
    ///
    /// # Errors
    ///
    /// Returns `AESError` if an error occurs during decryption.
    pub fn decrypt(&self, encrypted_message: String) -> Result<String, AESError> {
        let decoded_message = STANDARD.decode(encrypted_message).map_err(AESError::InvalidBase64)?;
        let blocks = Self::make_blocks(decoded_message);

        let decrypted_blocks: Vec<[[u8; 4]; 4]> = blocks
            .into_par_iter()
            .map(|block| self.decrypt_block(block))
            .collect();

        let flattened: Vec<u8> = decrypted_blocks
            .into_iter()
            .flat_map(|array4x4| array4x4.into_iter().flat_map(|array4| array4.into_iter()))
            .collect();

        let depadded_message = Self::depad(flattened);
        String::from_utf8(depadded_message).map_err(AESError::InvalidUTF8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_key() {
        let key = [0u8; 16];
        let aesteve = AESteve::new(&key).unwrap();
        assert_eq!(aesteve.keys[0][0], [0, 0, 0, 0]);
    }

    #[test]
    fn test_encrypt_decrypt() {
        let key = [0u8; 16];
        let aesteve = AESteve::new(&key).unwrap();
        let message = String::from("This is a test!");

        let encrypted_message = aesteve.encrypt(message.clone()).unwrap();
        let decrypted_message = aesteve.decrypt(encrypted_message).unwrap();

        assert_eq!(decrypted_message, message);
    }

    #[test]
    fn test_invalid_key_length() {
        let key = [0u8; 15];
        let result = AESteve::new(&key);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AESError::InvalidKeyLength));
    }
}