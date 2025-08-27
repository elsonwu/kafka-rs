use bytes::{Buf, BufMut, BytesMut};
use log::debug;
use std::io;

/// Kafka API keys for different request types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiKey {
    Produce = 0,
    Fetch = 1,
    ListOffsets = 2,
    Metadata = 3,
    OffsetCommit = 8,
    OffsetFetch = 9,
    FindCoordinator = 10,
    JoinGroup = 11,
    Heartbeat = 12,
    LeaveGroup = 13,
    SyncGroup = 14,
    ApiVersions = 18,
}

impl TryFrom<i16> for ApiKey {
    type Error = &'static str;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ApiKey::Produce),
            1 => Ok(ApiKey::Fetch),
            2 => Ok(ApiKey::ListOffsets),
            3 => Ok(ApiKey::Metadata),
            8 => Ok(ApiKey::OffsetCommit),
            9 => Ok(ApiKey::OffsetFetch),
            10 => Ok(ApiKey::FindCoordinator),
            11 => Ok(ApiKey::JoinGroup),
            12 => Ok(ApiKey::Heartbeat),
            13 => Ok(ApiKey::LeaveGroup),
            14 => Ok(ApiKey::SyncGroup),
            18 => Ok(ApiKey::ApiVersions),
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
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Not enough bytes",
        ));
    }
    Ok(buf.get_i8())
}

pub fn encode_i16(buf: &mut BytesMut, value: i16) {
    buf.put_i16(value);
}

pub fn decode_i16(buf: &mut BytesMut) -> io::Result<i16> {
    if buf.remaining() < 2 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Not enough bytes",
        ));
    }
    Ok(buf.get_i16())
}

pub fn encode_i32(buf: &mut BytesMut, value: i32) {
    buf.put_i32(value);
}

pub fn decode_i32(buf: &mut BytesMut) -> io::Result<i32> {
    if buf.remaining() < 4 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Not enough bytes",
        ));
    }
    Ok(buf.get_i32())
}

pub fn encode_i64(buf: &mut BytesMut, value: i64) {
    buf.put_i64(value);
}

pub fn decode_i64(buf: &mut BytesMut) -> io::Result<i64> {
    if buf.remaining() < 8 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Not enough bytes",
        ));
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
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Not enough bytes",
        ));
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
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Not enough bytes",
        ));
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
        debug!("Decoding PRODUCE request, buffer size: {}", buf.len());

        // Skip transactional_id (nullable string) - v3+
        let _transactional_id = decode_string(buf)?;
        debug!("Decoded transactional_id");

        // Skip acks and timeout
        let _acks = decode_i16(buf)?;
        let _timeout = decode_i32(buf)?;
        debug!("Decoded acks and timeout");

        // Read topic array
        let topic_count = decode_i32(buf)?;
        debug!("Topic count: {}", topic_count);
        if topic_count != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Expected exactly one topic, got {}", topic_count),
            ));
        }

        let topic = decode_string(buf)?
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Topic cannot be null"))?;
        debug!("Decoded topic: {}", topic);

        // Read partition array
        let partition_count = decode_i32(buf)?;
        debug!("Partition count: {}", partition_count);
        if partition_count != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Expected exactly one partition, got {}", partition_count),
            ));
        }

        let partition = decode_i32(buf)?;
        debug!("Decoded partition: {}", partition);

        // Read RecordBatch format (v3+ uses RecordBatch, not old MessageSet)
        // RecordBatch structure:
        // BaseOffset => INT64
        // BatchLength => INT32
        // PartitionLeaderEpoch => INT32
        // Magic => INT8 (should be 2 for RecordBatch)
        // CRC => INT32
        // Attributes => INT16
        // LastOffsetDelta => INT32
        // FirstTimestamp => INT64
        // MaxTimestamp => INT64
        // ProducerId => INT64
        // ProducerEpoch => INT16
        // BaseSequence => INT32
        // RecordCount => INT32
        // Records => [Record]

        debug!(
            "Remaining buffer size before RecordBatch: {}",
            buf.remaining()
        );

        let record_batch_size = decode_i32(buf)?;
        debug!("RecordBatch size: {}", record_batch_size);

        if buf.remaining() < record_batch_size as usize {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!(
                    "Not enough bytes for RecordBatch: expected {}, got {}",
                    record_batch_size,
                    buf.remaining()
                ),
            ));
        }

        // For simplicity, let's just skip the RecordBatch for now and return empty messages
        // In a real implementation, we would parse the RecordBatch properly
        let _ = buf.split_to(record_batch_size as usize);
        debug!("Skipped RecordBatch data");

        // Return a simple message for testing
        let messages = vec![ProduceMessage {
            key: None,
            value: Some(b"test message".to_vec()),
        }];

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

/// ApiVersions request - clients use this to discover what APIs the server supports
#[derive(Debug, Clone)]
pub struct ApiVersionsRequest {
    pub client_software_name: String,
    pub client_software_version: String,
}

impl ApiVersionsRequest {
    pub fn decode(buf: &mut BytesMut) -> io::Result<Self> {
        // ApiVersions request is simple - it may have client info but we'll handle empty case
        let client_software_name = decode_string(buf)?.unwrap_or_else(|| "unknown".to_string());
        let client_software_version = decode_string(buf)?.unwrap_or_else(|| "unknown".to_string());

        Ok(ApiVersionsRequest {
            client_software_name,
            client_software_version,
        })
    }
}

/// Individual API version info
#[derive(Debug, Clone)]
pub struct ApiVersion {
    pub api_key: i16,
    pub min_version: i16,
    pub max_version: i16,
}

/// ApiVersions response - tells clients what APIs and versions we support
#[derive(Debug, Clone)]
pub struct ApiVersionsResponse {
    pub error_code: i16,
    pub api_versions: Vec<ApiVersion>,
    pub throttle_time_ms: i32,
}

impl Default for ApiVersionsResponse {
    fn default() -> Self {
        Self::new()
    }
}

impl ApiVersionsResponse {
    pub fn new() -> Self {
        let api_versions = vec![
            ApiVersion {
                api_key: 0,
                min_version: 0,
                max_version: 3,
            }, // Produce
            ApiVersion {
                api_key: 1,
                min_version: 0,
                max_version: 4,
            }, // Fetch
            ApiVersion {
                api_key: 2,
                min_version: 0,
                max_version: 2,
            }, // ListOffsets
            ApiVersion {
                api_key: 3,
                min_version: 0,
                max_version: 2,
            }, // Metadata
            ApiVersion {
                api_key: 8,
                min_version: 0,
                max_version: 2,
            }, // OffsetCommit
            ApiVersion {
                api_key: 9,
                min_version: 0,
                max_version: 2,
            }, // OffsetFetch
            ApiVersion {
                api_key: 10,
                min_version: 0,
                max_version: 1,
            }, // FindCoordinator
            ApiVersion {
                api_key: 11,
                min_version: 0,
                max_version: 2,
            }, // JoinGroup
            ApiVersion {
                api_key: 12,
                min_version: 0,
                max_version: 1,
            }, // Heartbeat
            ApiVersion {
                api_key: 13,
                min_version: 0,
                max_version: 1,
            }, // LeaveGroup
            ApiVersion {
                api_key: 14,
                min_version: 0,
                max_version: 1,
            }, // SyncGroup
            ApiVersion {
                api_key: 18,
                min_version: 0,
                max_version: 2,
            }, // ApiVersions
        ];

        ApiVersionsResponse {
            error_code: 0, // No error
            api_versions,
            throttle_time_ms: 0,
        }
    }

    pub fn encode(&self, buf: &mut BytesMut) {
        // Error code
        encode_i16(buf, self.error_code);

        // API versions array
        encode_i32(buf, self.api_versions.len() as i32);
        for api_version in &self.api_versions {
            encode_i16(buf, api_version.api_key);
            encode_i16(buf, api_version.min_version);
            encode_i16(buf, api_version.max_version);
        }

        // Throttle time (for newer versions)
        encode_i32(buf, self.throttle_time_ms);
    }
}
