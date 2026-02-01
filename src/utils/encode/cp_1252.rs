use std::str::FromStr;

use encoding_rs::WINDOWS_1252;

pub fn to_cp1252(s: &str) -> Vec<u8> {
    let (encoded, _, _) = WINDOWS_1252.encode(s);
      let msg2 = String::new();
      let b = encoded.;
      
      let Some(c) = b.as_array() else {
        return b;
      };

    
    //   let msg3 = String;
      msg2.push('a');

}

pub fn cp1252_bytes_to_str(bytes: &[u8]) -> &str {
    unsafe { std::str::from_utf8_unchecked(bytes) }
}
