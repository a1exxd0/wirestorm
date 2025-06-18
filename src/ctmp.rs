const MAX_MSG_LEN: usize = 0xFFFF;
const SMALL_MSG_THRESHOLD: usize = 64;

/// The protocol specifies that each message contains
/// a static magic byte, the data length, and the data
/// itself.
///
/// A message is deemed invalid if the data inside of the message
/// does not match header length specified
#[derive(Clone, PartialEq, Debug)]
pub struct Ctmp {
    length: u16,
    data: CtmpData,
}

#[derive(Clone, PartialEq, Debug)]
enum CtmpData {
    Small(([u8; SMALL_MSG_THRESHOLD], u8)),
    Large(Vec<u8>),
}

impl Ctmp {
    #[inline(always)]
    pub fn new(data: &[u8]) -> Self {
        // Validate length doesn't exceed maximum
        let valid_len = valid_len.min(MAX_MSG_LEN as u16);
        let actual_len = data.len().min(valid_len as usize);
        
        if valid_len <= SMALL_MSG_THRESHOLD as u16 {
            let mut small_data = [0u8; SMALL_MSG_THRESHOLD];
            small_data[..actual_len].copy_from_slice(&data[..actual_len]);
            
            Ctmp {
                length: valid_len,
                data: CtmpData::Small(small_data),
            }
        } else {
            Ctmp {
                length: valid_len,
                data: CtmpData::Large(data[..actual_len].to_vec()),
            }
        }
    }
    
    /// Get the actual length of valid data
    pub fn len(&self) -> u16 {
        self.length
    }
    
    /// Check if the message is empty
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
    
    /// Get a slice of the valid data
    pub fn data(&self) -> &[u8] {
        match &self.data {
            CtmpData::Small(arr) => &arr[..self.length as usize],
            CtmpData::Large(vec) => &vec[..self.length as usize],
        }
    }
    
    /// Get mutable access to the valid data
    pub fn data_mut(&mut self) -> &mut [u8] {
        match &mut self.data {
            CtmpData::Small(arr) => &mut arr[..self.length as usize],
            CtmpData::Large(vec) => &mut vec[..self.length as usize],
        }
    }
    
    /// Check if this message uses small buffer optimization
    pub fn is_small(&self) -> bool {
        matches!(self.data, CtmpData::Small(_))
    }
}

impl Default for Ctmp {
    fn default() -> Self {
        Ctmp {
            length: 0,
            data: CtmpData::Small([0; SMALL_MSG_THRESHOLD]),
        }
    }
}

impl AsRef<[u8]> for Ctmp {
    fn as_ref(&self) -> &[u8] {
        self.data()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_message() {
        let data = b"hello world";
        let msg = Ctmp::new(data.len() as u16, data);
        
        assert_eq!(msg.len(), data.len() as u16);
        assert_eq!(msg.data(), data);
        assert!(msg.is_small());
    }
    
    #[test]
    fn test_large_message() {
        let data = vec![42u8; 100]; // Larger than 64 bytes
        let msg = Ctmp::new(data.len() as u16, &data);
        
        assert_eq!(msg.len(), data.len() as u16);
        assert_eq!(msg.data(), data.as_slice());
        assert!(!msg.is_small());
    }
    
    #[test]
    fn test_truncation() {
        let data = vec![1u8; 100];
        let msg = Ctmp::new(50, &data); // Shorter than actual data
        
        assert_eq!(msg.len(), 50);
        assert_eq!(msg.data().len(), 50);
        assert_eq!(msg.data(), &vec![1u8; 50]);
    }
    
    #[test]
    fn test_from_conversions() {
        let data = b"test data";
        let msg1 = Ctmp::from(data.as_slice());
        let msg2 = Ctmp::from(data.to_vec());
        
        assert_eq!(msg1.data(), data);
        assert_eq!(msg2.data(), data);
    }
    
    #[test]
    fn test_empty_message() {
        let msg = Ctmp::default();
        assert!(msg.is_empty());
        assert_eq!(msg.len(), 0);
        assert_eq!(msg.data().len(), 0);
    }
}