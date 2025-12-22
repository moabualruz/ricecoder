use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use memmap2::{Mmap, MmapOptions};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::chunking::{Chunk, ChunkMetadata, LanguageKind};

const VERSION: u32 = 1;
const HEADER_SIZE: usize = std::mem::size_of::<MetadataHeader>();
const RECORD_SIZE: usize = std::mem::size_of::<ChunkRecord>();

#[derive(Debug)]
#[repr(C)]
pub struct MetadataHeader {
    pub version: u32,
    pub reserved: u32,
    pub entry_count: u64,
    pub string_table_size: u64,
    pub entries_offset: u64,
    pub checksum: u64,
}

impl MetadataHeader {
    fn to_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut buf = [0u8; HEADER_SIZE];
        buf[0..4].copy_from_slice(&self.version.to_le_bytes());
        buf[4..8].copy_from_slice(&self.reserved.to_le_bytes());
        buf[8..16].copy_from_slice(&self.entry_count.to_le_bytes());
        buf[16..24].copy_from_slice(&self.string_table_size.to_le_bytes());
        buf[24..32].copy_from_slice(&self.entries_offset.to_le_bytes());
        buf[32..40].copy_from_slice(&self.checksum.to_le_bytes());
        buf
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, MetadataError> {
        if bytes.len() < HEADER_SIZE {
            return Err(MetadataError::Integrity("header too short".into()));
        }
        let version = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        let reserved = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
        let entry_count = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        let string_table_size = u64::from_le_bytes(bytes[16..24].try_into().unwrap());
        let entries_offset = u64::from_le_bytes(bytes[24..32].try_into().unwrap());
        let checksum = u64::from_le_bytes(bytes[32..40].try_into().unwrap());
        Ok(Self {
            version,
            reserved,
            entry_count,
            string_table_size,
            entries_offset,
            checksum,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ChunkRecord {
    pub chunk_id: u64,
    pub repository_id: Option<u32>,
    pub file_path_offset: u32,
    pub language_offset: u32,
    pub start_line: u32,
    pub end_line: u32,
    pub token_count: u32,
    pub checksum: u64,
}

impl ChunkRecord {
    fn to_bytes(&self) -> [u8; RECORD_SIZE] {
        let mut buf = [0u8; RECORD_SIZE];
        buf[0..8].copy_from_slice(&self.chunk_id.to_le_bytes());
        buf[8..12].copy_from_slice(
            &self
                .repository_id
                .map(|id| id as i32)
                .unwrap_or(-1)
                .to_le_bytes(),
        );
        buf[12..16].copy_from_slice(&self.file_path_offset.to_le_bytes());
        buf[16..20].copy_from_slice(&self.language_offset.to_le_bytes());
        buf[20..24].copy_from_slice(&self.start_line.to_le_bytes());
        buf[24..28].copy_from_slice(&self.end_line.to_le_bytes());
        buf[28..32].copy_from_slice(&self.token_count.to_le_bytes());
        buf[32..40].copy_from_slice(&self.checksum.to_le_bytes());
        buf
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, MetadataError> {
        if bytes.len() < RECORD_SIZE {
            return Err(MetadataError::Integrity("record too short".into()));
        }
        let chunk_id = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
        let repository = i32::from_le_bytes(bytes[8..12].try_into().unwrap());
        let file_path_offset = u32::from_le_bytes(bytes[12..16].try_into().unwrap());
        let language_offset = u32::from_le_bytes(bytes[16..20].try_into().unwrap());
        let start_line = u32::from_le_bytes(bytes[20..24].try_into().unwrap());
        let end_line = u32::from_le_bytes(bytes[24..28].try_into().unwrap());
        let token_count = u32::from_le_bytes(bytes[28..32].try_into().unwrap());
        let checksum = u64::from_le_bytes(bytes[32..40].try_into().unwrap());
        Ok(Self {
            chunk_id,
            repository_id: if repository >= 0 {
                Some(repository as u32)
            } else {
                None
            },
            file_path_offset,
            language_offset,
            start_line,
            end_line,
            token_count,
            checksum,
        })
    }
}

fn compute_text_checksum(text: &str) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let digest = hasher.finalize();
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&digest[..8]);
    u64::from_le_bytes(bytes)
}

const CHUNK_MAP_ENTRY_SIZE: usize = std::mem::size_of::<ChunkMapEntry>();
const DELTA_VERSION: u32 = 1;
const DELTA_HEADER_SIZE: usize = std::mem::size_of::<DeltaHeader>();

#[repr(C)]
struct ChunkMapEntry {
    chunk_id: u64,
    offset: u64,
}

enum RecordPointer {
    Base(usize),
    Delta(usize),
}

#[repr(C)]
struct DeltaHeader {
    version: u32,
    reserved: u32,
    entry_count: u64,
}

impl DeltaHeader {
    fn new(entry_count: u64) -> Self {
        Self {
            version: DELTA_VERSION,
            reserved: 0,
            entry_count,
        }
    }

    fn to_bytes(&self) -> [u8; DELTA_HEADER_SIZE] {
        let mut buf = [0u8; DELTA_HEADER_SIZE];
        buf[0..4].copy_from_slice(&self.version.to_le_bytes());
        buf[4..8].copy_from_slice(&self.reserved.to_le_bytes());
        buf[8..16].copy_from_slice(&self.entry_count.to_le_bytes());
        buf
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, MetadataError> {
        if bytes.len() < DELTA_HEADER_SIZE {
            return Err(MetadataError::Integrity("delta header too short".into()));
        }
        let version = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        let reserved = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
        let entry_count = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        Ok(Self {
            version,
            reserved,
            entry_count,
        })
    }
}

struct DeltaEntry {
    record: ChunkRecord,
    file_path: String,
    language: String,
}

impl DeltaEntry {
    fn to_view(&self) -> ChunkMetadataView {
        ChunkMetadataView {
            chunk_id: self.record.chunk_id,
            repository_id: self.record.repository_id,
            file_path: self.file_path.clone(),
            language: self.language.clone(),
            start_line: self.record.start_line,
            end_line: self.record.end_line,
            token_count: self.record.token_count,
            checksum: self.record.checksum,
        }
    }
}

struct DeltaStore {
    entries: Vec<DeltaEntry>,
}

impl DeltaStore {
    fn load(base_path: &Path) -> Result<Option<Self>, MetadataError> {
        let delta_path = delta_path(base_path);
        if !delta_path.exists() {
            return Ok(None);
        }
        let mut file = File::open(&delta_path)?;
        let mut header_buf = [0u8; DELTA_HEADER_SIZE];
        file.read_exact(&mut header_buf)?;
        let header = DeltaHeader::from_bytes(&header_buf)?;
        if header.version != DELTA_VERSION {
            return Err(MetadataError::Integrity("unsupported delta version".into()));
        }
        let mut entries = Vec::with_capacity(header.entry_count as usize);
        for _ in 0..header.entry_count {
            let mut record_buf = [0u8; RECORD_SIZE];
            file.read_exact(&mut record_buf)?;
            let record = ChunkRecord::from_bytes(&record_buf)?;
            let file_path = read_length_prefixed_string(&mut file)?;
            let language = read_length_prefixed_string(&mut file)?;
            entries.push(DeltaEntry {
                record,
                file_path,
                language,
            });
        }
        Ok(Some(Self { entries }))
    }
}

pub struct MetadataAppender {
    file: File,
    path: PathBuf,
    header: DeltaHeader,
}

impl MetadataAppender {
    pub fn open(base_path: &Path) -> Result<Self, MetadataError> {
        let delta_path = delta_path(base_path);
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&delta_path)?;
        let metadata = file.metadata()?;
        let header = if metadata.len() >= DELTA_HEADER_SIZE as u64 {
            let mut header_buf = [0u8; DELTA_HEADER_SIZE];
            file.read_exact(&mut header_buf)?;
            let header = DeltaHeader::from_bytes(&header_buf)?;
            file.seek(SeekFrom::End(0))?;
            header
        } else {
            let header = DeltaHeader::new(0);
            file.write_all(&header.to_bytes())?;
            file.sync_all()?;
            header
        };
        Ok(Self {
            file,
            path: delta_path,
            header,
        })
    }

    pub fn append_chunk(&mut self, chunk: &Chunk) -> Result<(), MetadataError> {
        let metadata = &chunk.metadata;
        let file_path = metadata.file_path.to_string_lossy();
        let language = format!("{:?}", chunk.language);
        let record = ChunkRecord {
            chunk_id: chunk.id,
            repository_id: metadata.repository_id,
            file_path_offset: 0,
            language_offset: 0,
            start_line: metadata.start_line,
            end_line: metadata.end_line,
            token_count: metadata.token_count,
            checksum: compute_text_checksum(&chunk.text),
        };
        self.file.seek(SeekFrom::End(0))?;
        self.file.write_all(&record.to_bytes())?;
        write_length_prefixed_string(&mut self.file, file_path.as_bytes())?;
        write_length_prefixed_string(&mut self.file, language.as_bytes())?;
        self.header.entry_count += 1;
        self.file.seek(SeekFrom::Start(0))?;
        self.file.write_all(&self.header.to_bytes())?;
        self.file.sync_all()?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum MetadataError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Integrity violation: {0}")]
    Integrity(String),
}

pub struct MetadataWriter {
    records: Vec<ChunkRecord>,
    string_table: Vec<u8>,
    string_offsets: HashMap<String, u32>,
}

impl MetadataWriter {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            string_table: Vec::new(),
            string_offsets: HashMap::new(),
        }
    }

    pub fn add_chunk(&mut self, chunk: &Chunk) {
        let metadata = &chunk.metadata;
        let file_path = metadata.file_path.to_string_lossy().to_string();
        let language = format!("{:?}", chunk.language);
        let file_offset = self.add_string(&file_path);
        let language_offset = self.add_string(&language);
        let entry = ChunkRecord {
            chunk_id: chunk.id,
            repository_id: metadata.repository_id,
            file_path_offset: file_offset,
            language_offset,
            start_line: metadata.start_line,
            end_line: metadata.end_line,
            token_count: metadata.token_count,
            checksum: compute_text_checksum(&chunk.text),
        };
        self.records.push(entry);
    }

    fn add_string(&mut self, value: &str) -> u32 {
        if let Some(&offset) = self.string_offsets.get(value) {
            return offset;
        }
        let offset = self.string_table.len() as u32;
        self.string_table
            .extend_from_slice(&(value.len() as u32).to_le_bytes());
        self.string_table.extend_from_slice(value.as_bytes());
        self.string_offsets.insert(value.to_string(), offset);
        offset
    }

    pub fn finalize(&self, path: &Path) -> Result<(), MetadataError> {
        let mut file = File::create(path)?;
        let header_size = HEADER_SIZE as u64;
        let string_table_size = self.string_table.len() as u64;
        let entries_offset = header_size + string_table_size;
        let checksum = self.compute_checksum();
        let header = MetadataHeader {
            version: VERSION,
            reserved: 0,
            entry_count: self.records.len() as u64,
            string_table_size,
            entries_offset,
            checksum,
        };
        file.write_all(&header.to_bytes())?;
        file.write_all(&self.string_table)?;
        for record in &self.records {
            file.write_all(&record.to_bytes())?;
        }
        file.sync_all()?;
        write_supplementary_files(path, &header, &self.records, &self.string_table)?;
        Ok(())
    }

    fn compute_checksum(&self) -> u64 {
        let mut hasher = Sha256::new();
        hasher.update(&self.string_table);
        for record in &self.records {
            hasher.update(&record.to_bytes());
        }
        let digest = hasher.finalize();
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&digest[..8]);
        u64::from_le_bytes(bytes)
    }
}

pub struct MetadataStore {
    mmap: Arc<Mmap>,
    header: MetadataHeader,
    string_table_offset: usize,
    string_table_size: usize,
    records_offset: usize,
    chunk_index: HashMap<u64, RecordPointer>,
    delta: Option<DeltaStore>,
}

impl MetadataStore {
    pub fn load(path: &Path) -> Result<Self, MetadataError> {
        let file = File::open(path)?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        let header = MetadataHeader::from_bytes(&mmap[..HEADER_SIZE])?;
        if header.version != VERSION {
            return Err(MetadataError::Integrity("unsupported version".into()));
        }
        let string_table_offset = HEADER_SIZE;
        let string_table_size = header.string_table_size as usize;
        let records_offset = header.entries_offset as usize;
        if string_table_offset + string_table_size > mmap.len() {
            return Err(MetadataError::Integrity("string table trimmed".into()));
        }
        let expected_records_size = (header.entry_count as usize)
            .checked_mul(RECORD_SIZE)
            .ok_or_else(|| MetadataError::Integrity("record table length overflow".into()))?;
        if records_offset + expected_records_size > mmap.len() {
            return Err(MetadataError::Integrity("record table truncated".into()));
        }
        let computed = MetadataStore::compute_checksum(
            &mmap,
            header.entry_count as usize,
            string_table_size,
            records_offset,
        )?;
        if computed != header.checksum {
            return Err(MetadataError::Integrity("checksum mismatch".into()));
        }
        let mut chunk_index =
            Self::build_chunk_index(&mmap, path, records_offset, header.entry_count as usize)?;
        let delta = DeltaStore::load(path)?;
        if let Some(delta_store) = &delta {
            for (idx, entry) in delta_store.entries.iter().enumerate() {
                chunk_index.insert(entry.record.chunk_id, RecordPointer::Delta(idx));
            }
        }
        Ok(Self {
            mmap: Arc::new(mmap),
            header,
            string_table_offset,
            string_table_size,
            records_offset,
            chunk_index,
            delta,
        })
    }

    fn build_chunk_index(
        mmap: &[u8],
        path: &Path,
        records_offset: usize,
        entry_count: usize,
    ) -> Result<HashMap<u64, RecordPointer>, MetadataError> {
        let map_path = chunk_map_path(path);
        if let Ok(contents) = std::fs::read(&map_path) {
            if contents.len() % CHUNK_MAP_ENTRY_SIZE != 0 {
                return Err(MetadataError::Integrity(
                    "chunk map has invalid entry size".into(),
                ));
            }
            let mut index = HashMap::with_capacity(entry_count);
            for chunk_bytes in contents.chunks_exact(CHUNK_MAP_ENTRY_SIZE) {
                let chunk_id = u64::from_le_bytes(chunk_bytes[0..8].try_into().unwrap());
                let offset = u64::from_le_bytes(chunk_bytes[8..16].try_into().unwrap());
                if offset < records_offset as u64 {
                    return Err(MetadataError::Integrity(
                        "chunk map offset below record table".into(),
                    ));
                }
                let relative = offset - records_offset as u64;
                if relative % RECORD_SIZE as u64 != 0 {
                    return Err(MetadataError::Integrity(
                        "chunk map offset misaligned".into(),
                    ));
                }
                let idx = (relative / RECORD_SIZE as u64) as usize;
                index.insert(chunk_id, RecordPointer::Base(idx));
            }
            return Ok(index);
        }
        let mut fallback = HashMap::with_capacity(entry_count);
        for idx in 0..entry_count {
            let offset = records_offset + idx * RECORD_SIZE;
            let record = ChunkRecord::from_bytes(&mmap[offset..offset + RECORD_SIZE])?;
            fallback.insert(record.chunk_id, RecordPointer::Base(idx));
        }
        Ok(fallback)
    }

    fn compute_checksum(
        mmap: &[u8],
        entry_count: usize,
        string_size: usize,
        records_offset: usize,
    ) -> Result<u64, MetadataError> {
        let mut hasher = Sha256::new();
        let start = HEADER_SIZE;
        let end = start + string_size;
        if end > mmap.len() {
            return Err(MetadataError::Integrity(
                "string table out of bounds".into(),
            ));
        }
        hasher.update(&mmap[start..end]);
        for idx in 0..entry_count {
            let offset = records_offset + idx * RECORD_SIZE;
            if offset + RECORD_SIZE > mmap.len() {
                return Err(MetadataError::Integrity("record table truncated".into()));
            }
            hasher.update(&mmap[offset..offset + RECORD_SIZE]);
        }
        let digest = hasher.finalize();
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&digest[..8]);
        Ok(u64::from_le_bytes(bytes))
    }

    pub fn get(&self, chunk_id: u64) -> Result<ChunkMetadataView, MetadataError> {
        match self.chunk_index.get(&chunk_id) {
            Some(RecordPointer::Base(idx)) => self.get_base_view(*idx),
            Some(RecordPointer::Delta(idx)) => self.get_delta_view(*idx),
            None => Err(MetadataError::Integrity("chunk not found".into())),
        }
    }

    fn get_base_view(&self, idx: usize) -> Result<ChunkMetadataView, MetadataError> {
        let offset = self.records_offset + idx * RECORD_SIZE;
        let record = ChunkRecord::from_bytes(&self.mmap[offset..offset + RECORD_SIZE])?;
        Ok(self.view(record))
    }

    fn get_delta_view(&self, idx: usize) -> Result<ChunkMetadataView, MetadataError> {
        let delta = self
            .delta
            .as_ref()
            .ok_or_else(|| MetadataError::Integrity("delta store missing".into()))?;
        delta
            .entries
            .get(idx)
            .map(|entry| entry.to_view())
            .ok_or_else(|| MetadataError::Integrity("delta record out of range".into()))
    }

    fn view(&self, record: ChunkRecord) -> ChunkMetadataView {
        let file_path = self.decode_string(record.file_path_offset);
        let language = self.decode_string(record.language_offset);
        ChunkMetadataView {
            chunk_id: record.chunk_id,
            repository_id: record.repository_id,
            file_path,
            language,
            start_line: record.start_line,
            end_line: record.end_line,
            token_count: record.token_count,
            checksum: record.checksum,
        }
    }

    fn decode_string(&self, offset: u32) -> String {
        let base = self.string_table_offset + offset as usize;
        if base + 4 > self.mmap.len() {
            return String::new();
        }
        let len_bytes = &self.mmap[base..base + 4];
        let len = u32::from_le_bytes(len_bytes.try_into().unwrap()) as usize;
        let start = base + 4;
        let end = start + len;
        if end > self.mmap.len() {
            return String::new();
        }
        String::from_utf8_lossy(&self.mmap[start..end]).into_owned()
    }
}

pub struct ChunkMetadataView {
    pub chunk_id: u64,
    pub repository_id: Option<u32>,
    pub file_path: String,
    pub language: String,
    pub start_line: u32,
    pub end_line: u32,
    pub token_count: u32,
    pub checksum: u64,
}

fn write_supplementary_files(
    base_path: &Path,
    header: &MetadataHeader,
    records: &[ChunkRecord],
    string_table: &[u8],
) -> Result<(), MetadataError> {
    let chunk_map_path = chunk_map_path(base_path);
    let mut chunk_map = File::create(chunk_map_path)?;
    for (idx, record) in records.iter().enumerate() {
        chunk_map.write_all(&record.chunk_id.to_le_bytes())?;
        let offset = header.entries_offset + (idx * RECORD_SIZE) as u64;
        chunk_map.write_all(&offset.to_le_bytes())?;
    }
    chunk_map.sync_all()?;

    let offsets_path = sibling_path(base_path, "offsets.bin");
    let mut offsets = File::create(offsets_path)?;
    for record in records {
        offsets.write_all(&record.file_path_offset.to_le_bytes())?;
        offsets.write_all(&record.language_offset.to_le_bytes())?;
    }
    offsets.sync_all()?;

    let ranges_path = sibling_path(base_path, "ranges.bin");
    let mut ranges = File::create(ranges_path)?;
    for record in records {
        ranges.write_all(&record.start_line.to_le_bytes())?;
        ranges.write_all(&record.end_line.to_le_bytes())?;
    }
    ranges.sync_all()?;

    let path_table_path = sibling_path(base_path, "path_table.bin");
    let mut path_table = File::create(path_table_path)?;
    path_table.write_all(string_table)?;
    path_table.sync_all()?;

    let identifiers_path = sibling_path(base_path, "identifiers.bin");
    let mut identifiers = File::create(identifiers_path)?;
    identifiers.write_all(&0u32.to_le_bytes())?;
    identifiers.sync_all()?;

    Ok(())
}

fn sibling_path(base: &Path, suffix: &str) -> PathBuf {
    let parent = base.parent().unwrap_or_else(|| Path::new("."));
    let stem = base
        .file_stem()
        .and_then(OsStr::to_str)
        .map(|s| s.to_string())
        .or_else(|| {
            base.file_name()
                .and_then(OsStr::to_str)
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "metadata".to_string());
    parent.join(format!("{stem}.{suffix}"))
}

fn chunk_map_path(base: &Path) -> PathBuf {
    sibling_path(base, "chunks.map")
}

fn delta_path(base: &Path) -> PathBuf {
    sibling_path(base, "delta")
}

fn write_length_prefixed_string(
    writer: &mut impl Write,
    bytes: &[u8],
) -> Result<(), MetadataError> {
    if bytes.len() > u32::MAX as usize {
        return Err(MetadataError::Integrity("string too long".into()));
    }
    writer.write_all(&(bytes.len() as u32).to_le_bytes())?;
    writer.write_all(bytes)?;
    Ok(())
}

fn read_length_prefixed_string(reader: &mut impl Read) -> Result<String, MetadataError> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf)?;
    let len = u32::from_le_bytes(len_buf) as usize;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tempfile::NamedTempFile;

    use super::*;
    use crate::chunking::{ChunkMetadata, LanguageKind};

    fn stub_chunk(id: u64) -> Chunk {
        let path = PathBuf::from(format!("chunk_{id}.rs"));
        let metadata = ChunkMetadata {
            chunk_id: id,
            repository_id: Some(1),
            file_path: path.clone(),
            language: LanguageKind::Rust,
            start_line: 1,
            end_line: 1,
            token_count: 3,
            checksum: format!("sha{id}"),
        };
        Chunk {
            id,
            language: LanguageKind::Rust,
            file_path: path.clone(),
            start_line: 1,
            end_line: 1,
            text: "fn test() {}".to_string(),
            identifiers: vec!["test".into()],
            identifier_tokens: vec!["test".into()],
            metadata,
        }
    }

    #[test]
    fn writer_roundtrip_exports_chunks() -> Result<(), MetadataError> {
        let mut writer = MetadataWriter::new();
        let chunk_a = stub_chunk(1);
        let chunk_b = stub_chunk(2);
        writer.add_chunk(&chunk_a);
        writer.add_chunk(&chunk_b);
        let file = NamedTempFile::new()?;
        writer.finalize(file.path())?;
        let store = MetadataStore::load(file.path())?;
        let view = store.get(chunk_a.id)?;
        assert_eq!(view.file_path, "chunk_1.rs");
        assert_eq!(view.language, "Rust");
        assert_eq!(view.token_count, chunk_a.metadata.token_count);
        Ok(())
    }

    #[test]
    fn bad_checksum_rejected() -> Result<(), MetadataError> {
        let mut writer = MetadataWriter::new();
        writer.add_chunk(&stub_chunk(1));
        let temp = NamedTempFile::new()?;
        writer.finalize(temp.path())?;
        std::fs::OpenOptions::new()
            .write(true)
            .open(temp.path())?
            .write_all(b"corrupt")?;
        assert!(MetadataStore::load(temp.path()).is_err());
        Ok(())
    }

    #[test]
    fn missing_chunk_returns_error() -> Result<(), MetadataError> {
        let mut writer = MetadataWriter::new();
        writer.add_chunk(&stub_chunk(1));
        let temp = NamedTempFile::new()?;
        writer.finalize(temp.path())?;
        let store = MetadataStore::load(temp.path())?;
        assert!(store.get(999).is_err());
        Ok(())
    }

    #[test]
    fn incremental_metadata_appends_visible() -> Result<(), MetadataError> {
        let mut writer = MetadataWriter::new();
        writer.add_chunk(&stub_chunk(1));
        let file = NamedTempFile::new()?;
        writer.finalize(file.path())?;
        let mut appender = MetadataAppender::open(file.path())?;
        appender.append_chunk(&stub_chunk(2))?;
        let store = MetadataStore::load(file.path())?;
        assert_eq!(store.get(1)?.chunk_id, 1);
        assert_eq!(store.get(2)?.chunk_id, 2);
        Ok(())
    }
}
