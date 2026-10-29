//! Embedded format generated and verified by cargo xtask material-symbols.
use flate2::read::GzDecoder;
use iced_core::svg::Handle;
use std::{
    collections::{HashMap, VecDeque},
    io::Read,
    sync::{Mutex, OnceLock},
};

const BUNDLE: &[u8] = super::BUNDLE;
const HEADER_BYTES: usize = 20;
const RECORD_BYTES: usize = 12;
const CHUNK_BYTES: usize = 64 * 1024;
const CACHED_CHUNKS: usize = 16;
type Entry = [usize; 3];

fn word(offset: usize) -> usize {
    u32::from_le_bytes(BUNDLE[offset..offset + 4].try_into().unwrap()) as usize
}

fn record(offset: usize) -> Entry {
    [word(offset), word(offset + 4), word(offset + 8)]
}

fn entry(index: usize) -> Entry {
    assert!(index < word(8), "Invalid bundled symbol index");
    record(HEADER_BYTES + index * RECORD_BYTES)
}

fn decode(chunk: usize) -> Vec<u8> {
    assert!(chunk < word(12), "Invalid bundled chunk index");
    let [offset, length, decoded_length] = record(HEADER_BYTES + (word(8) + chunk) * RECORD_BYTES);
    assert!((1..=CHUNK_BYTES).contains(&decoded_length));
    let start = word(16) + offset;
    let mut bytes = vec![0; decoded_length];
    let mut decoder = GzDecoder::new(&BUNDLE[start..start + length]);
    decoder
        .read_exact(&mut bytes)
        .expect("Invalid embedded Material Symbols gzip stream");
    // Finish the stream to check its CRC and reject oversized decoded output.
    assert_eq!(
        decoder.read(&mut [0]).expect("Invalid embedded gzip CRC"),
        0
    );
    bytes
}

#[derive(Default)]
struct Cache {
    handles: HashMap<Entry, Handle>,
    // Front is most recently used. Handles own their SVG bytes, so eviction is safe.
    chunks: VecDeque<(usize, Vec<u8>)>,
}

impl Cache {
    fn handle(&mut self, index: usize) -> Handle {
        let key @ [chunk, offset, length] = entry(index);
        if let Some(handle) = self.handles.get(&key) {
            return handle.clone();
        }
        let bytes = if let Some(position) = self.chunks.iter().position(|(id, _)| *id == chunk) {
            self.chunks.remove(position).unwrap().1
        } else {
            decode(chunk)
        };
        let handle = Handle::from_memory(bytes[offset..offset + length].to_vec());
        if self.chunks.len() == CACHED_CHUNKS {
            self.chunks.pop_back();
        }
        self.chunks.push_front((chunk, bytes));
        self.handles.insert(key, handle.clone());
        handle
    }
}

pub(super) fn handle(index: usize) -> Handle {
    static CACHE: OnceLock<Mutex<Cache>> = OnceLock::new();
    CACHE
        .get_or_init(|| Mutex::new(Cache::default()))
        .lock()
        .expect("Material Symbols cache lock poisoned")
        .handle(index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced_core::svg::Data;

    #[test]
    fn eviction_keeps_handles_valid_and_respects_the_decoded_memory_budget() {
        assert_eq!(&BUNDLE[..8], b"M3SYM001");
        let mut cache = Cache::default();
        let first = cache.handle(0);
        let Data::Bytes(original) = first.data() else {
            panic!("Expected embedded bytes")
        };
        let first_chunk = entry(0)[0];
        for index in 0..word(8) {
            if entry(index)[0] > CACHED_CHUNKS {
                break;
            }
            cache.handle(index);
        }
        assert_eq!(cache.chunks.len(), CACHED_CHUNKS);
        assert!(cache.chunks.iter().all(|(id, _)| *id != first_chunk));
        assert!(
            cache
                .chunks
                .iter()
                .map(|(_, bytes)| bytes.len())
                .sum::<usize>()
                <= CACHED_CHUNKS * CHUNK_BYTES
        );
        assert!(std::ptr::eq(first.data(), cache.handle(0).data()));
        // Force decoding after eviction, with the old externally-owned handle alive.
        cache.handles.clear();
        let recovered = cache.handle(0);
        let Data::Bytes(bytes) = recovered.data() else {
            panic!("Expected embedded bytes")
        };
        assert_eq!(original.as_ref(), bytes.as_ref());
    }

    #[test]
    fn simultaneous_requests_share_the_same_handle_allocation() {
        let handles: Vec<_> = std::thread::scope(|scope| {
            let threads: Vec<_> = (0..8).map(|_| scope.spawn(|| handle(42))).collect();
            threads
                .into_iter()
                .map(|thread| thread.join().unwrap())
                .collect()
        });
        assert!(
            handles
                .iter()
                .all(|handle| std::ptr::eq(handle.data(), handles[0].data()))
        );
    }
}
