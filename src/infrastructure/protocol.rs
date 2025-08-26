use std::io;
use bytes::{Buf, BufMut, BytesMut};

/// Kafka API keys for different request types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiKey {
    Produce = 0,
    Fetch = 1,
    Metadata = 3,
    OffsetCommit = 8,
    OffsetFetch = 9,
    FindCoordinator = 10,
    JoinGroup = 11,
    Heartbeat = 12,
    LeaveGroup = 13,
    SyncGroup = 14,
}

impl TryFrom<i16> for ApiKey {
    type Error = &'static str;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ApiKey::Produce),
            1 => Ok(ApiKey::Fetch),
            3 => Ok(ApiKey::Metadata),
            8 => Ok(ApiKey::OffsetCommit),
            9 => Ok(ApiKey::OffsetFetch),
            10 => Ok(ApiKey::FindCoordinator),
            11 => Ok(ApiKey::JoinGroup),
            12 => Ok(ApiKey::Heartbeat),
            13 => Ok(ApiKey::LeaveGroup),
            14 => Ok(ApiKey::SyncGroup),
            _ => Err("Unknown API key"),
        }
    }
}

/// Kafka request header
#[derive(Debug, Clone)]
pub struct RequestHeader {
    pub api_key: ApiKey,
    pub api_version: i16,
    pub correlation_id: i32,
    pub client_id: Option<String>,
}

/// Kafka response header
#[derive(Debug, Clone)]
pub struct ResponseHeader {
    pub correlation_id: i32,
}

/// Trait for encoding Kafka protocol messages
pub trait KafkaEncodable {
    fn encode(&self, buf: &mut BytesMut) -> io::Result<()>;
}

/// Trait for decoding Kafka protocol messages
pub trait KafkaDecodable: Sized {
    fn decode(buf: &mut BytesMut) -> io::Result<Self>;
}

// Helper functions for encoding/decoding primitive types

pub fn encode_i8(buf: &mut BytesMut, value: i8) {
    buf.put_i8(value);
}

pub fn decode_i8(buf: &mut BytesMut) -> io::Result<i8> {
    if buf.remaining() < 1 {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Not enough bytes"));
    }
    Ok(buf.get_i8())
}

pub fn encode_i16(buf: &mut BytesMut, value: i16) {
    buf.put_i16(value);
}

pub fn decode_i16(buf: &mut BytesMut) -> io::Result<i16> {
    if buf.remaining() < 2 {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Not enough bytes"));
    }
    Ok(buf.get_i16())
}

pub fn encode_i32(buf: &mut BytesMut, value: i32) {
    buf.put_i32(value);
}

pub fn decode_i32(buf: &mut BytesMut) -> io::Result<i32> {
    if buf.remaining() < 4 {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Not enough bytes"));
    }
    Ok(buf.get_i32())
}

pub fn encode_i64(buf: &mut BytesMut, value: i64) {
    buf.put_i64(value);
}

pub fn decode_i64(buf: &mut BytesMut) -> io::Result<i64> {
    if buf.remaining() < 8 {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Not enough bytes"));
    }
    Ok(buf.get_i64())
}

pub fn encode_string(buf: &mut BytesMut, value: Option<&str>) -> io::Result<()> {
    match value {
        Some(s) => {
            let bytes = s.as_bytes();
            encode_i16(buf, bytes.len() as i16);
            buf.put_slice(bytes);
        }
        None => {
            encode_i16(buf, -1);
        }
    }
    Ok(())
}

pub fn decode_string(buf: &mut BytesMut) -> io::Result<Option<String>> {
    let len = decode_i16(buf)?;
    if len == -1 {
        return Ok(None);
    }
    if len < 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid string length",
        ));
    }
    if buf.remaining() < len as usize {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Not enough bytes"));
    }
    
    let bytes = buf.split_to(len as usize);
    let s = String::from_utf8(bytes.to_vec())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8"))?;
    Ok(Some(s))
}

pub fn encode_bytes(buf: &mut BytesMut, value: Option<&[u8]>) -> io::Result<()> {
    match value {
        Some(bytes) => {
            encode_i32(buf, bytes.len() as i32);
            buf.put_slice(bytes);
        }
        None => {
            encode_i32(buf, -1);
        }
    }
    Ok(())
}

pub fn decode_bytes(buf: &mut BytesMut) -> io::Result<Option<Vec<u8>>> {
    let len = decode_i32(buf)?;
    if len == -1 {
        return Ok(None);
    }
    if len < 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid bytes length",
        ));
    }
    if buf.remaining() < len as usize {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Not enough bytes"));
    }
    
    let bytes = buf.split_to(len as usize);
    Ok(Some(bytes.to_vec()))
}

impl KafkaEncodable for RequestHeader {
    fn encode(&self, buf: &mut BytesMut) -> io::Result<()> {
        encode_i16(buf, self.api_key as i16);
        encode_i16(buf, self.api_version);
        encode_i32(buf, self.correlation_id);
        encode_string(buf, self.client_id.as_deref())?;
        Ok(())
    }
}

impl KafkaDecodable for RequestHeader {
    fn decode(buf: &mut BytesMut) -> io::Result<Self> {
        let api_key_raw = decode_i16(buf)?;
        let api_key = ApiKey::try_from(api_key_raw)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Unknown API key"))?;
        let api_version = decode_i16(buf)?;
        let correlation_id = decode_i32(buf)?;
        let client_id = decode_string(buf)?;
        
        Ok(RequestHeader {
            api_key,
            api_version,
            correlation_id,
            client_id,
        })
    }
}

impl KafkaEncodable for ResponseHeader {
    fn encode(&self, buf: &mut BytesMut) -> io::Result<()> {
        encode_i32(buf, self.correlation_id);
        Ok(())
    }
}

impl KafkaDecodable for ResponseHeader {
    fn decode(buf: &mut BytesMut) -> io::Result<Self> {
        let correlation_id = decode_i32(buf)?;
        Ok(ResponseHeader { correlation_id })
    }
}

/// Simple produce request (simplified version)
#[derive(Debug, Clone)]
pub struct ProduceRequest {
    pub topic: String,
    pub partition: i32,
    pub messages: Vec<ProduceMessage>,
}

#[derive(Debug, Clone)]
pub struct ProduceMessage {
    pub key: Option<Vec<u8>>,
    pub value: Option<Vec<u8>>,
}

impl KafkaDecodable for ProduceRequest {
    fn decode(buf: &mut BytesMut) -> io::Result<Self> {
        // Skip transactional_id (nullable string)
        let _transactional_id = decode_string(buf)?;
        
        // Skip acks and timeout
        let _acks = decode_i16(buf)?;
        let _timeout = decode_i32(buf)?;
        
        // Read topic array
        let topic_count = decode_i32(buf)?;
        if topic_count != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Expected exactly one topic",
            ));
        }
        
        let topic = decode_string(buf)?
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Topic cannot be null"))?;
        
        // Read partition array
        let partition_count = decode_i32(buf)?;
        if partition_count != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Expected exactly one partition",
            ));
        }
        
        let partition = decode_i32(buf)?;
        
        // Read message set size
        let message_set_size = decode_i32(buf)?;
        let mut messages = Vec::new();
        
        // Simple message parsing (very basic)
        let mut remaining = message_set_size as usize;
        while remaining > 0 && buf.remaining() >= 8 {
            let _offset = decode_i64(buf)?;
            let message_size = decode_i32(buf)?;
            
            if message_size <= 0 || buf.remaining() < message_size as usize {
                break;
            }
            
            // Skip CRC
            let _crc = decode_i32(buf)?;
            
            // Skip magic byte and attributes
            let _magic = decode_i8(buf)?;
            let _attributes = decode_i8(buf)?;
            
            // Skip timestamp (if present)
            if buf.remaining() >= 8 {
                let _timestamp = decode_i64(buf)?;
            }
            
            // Read key and value
            let key = decode_bytes(buf)?;
            let value = decode_bytes(buf)?;
            
            messages.push(ProduceMessage { key, value });
            
            remaining = remaining.saturating_sub(12 + message_size as usize);
        }
        
        Ok(ProduceRequest {
            topic,
            partition,
            messages,
        })
    }
}

/// Simple fetch request
#[derive(Debug, Clone)]
pub struct FetchRequest {
    pub replica_id: i32,
    pub max_wait: i32,
    pub min_bytes: i32,
    pub topic: String,
    pub partition: i32,
    pub offset: i64,
    pub max_bytes: i32,
}

impl KafkaDecodable for FetchRequest {
    fn decode(buf: &mut BytesMut) -> io::Result<Self> {
        let replica_id = decode_i32(buf)?;
        let max_wait = decode_i32(buf)?;
        let min_bytes = decode_i32(buf)?;
        let _max_bytes_overall = decode_i32(buf)?;
        let _isolation_level = decode_i8(buf)?;
        let _session_id = decode_i32(buf)?;
        let _session_epoch = decode_i32(buf)?;
        
        // Read topics
        let topic_count = decode_i32(buf)?;
        if topic_count != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Expected exactly one topic",
            ));
        }
        
        let topic = decode_string(buf)?
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Topic cannot be null"))?;
        
        // Read partitions
        let partition_count = decode_i32(buf)?;
        if partition_count != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Expected exactly one partition",
            ));
        }
        
        let partition = decode_i32(buf)?;
        let _current_leader_epoch = decode_i32(buf)?;
        let offset = decode_i64(buf)?;
        let _log_start_offset = decode_i64(buf)?;
        let max_bytes = decode_i32(buf)?;
        
        Ok(FetchRequest {
            replica_id,
            max_wait,
            min_bytes,
            topic,
            partition,
            offset,
            max_bytes,
        })
    }
}
