//! Integration tests for the shared-memory force queue (Task 1-7)

#[cfg(test)]
mod tests {
    use mps_core::rapier::ffi::force_queue::ForceQueueHeader;

    #[test]
    fn force_queue_header_layout_matches_java() {
        // Cache-line aligned (64 bytes)
        assert_eq!(std::mem::size_of::<ForceQueueHeader>(), 64);
        assert_eq!(std::mem::align_of::<ForceQueueHeader>(), 64);
    }

    #[test]
    fn force_queue_header_bitmap_offset() {
        // capacity=1024 -> bitmap_words = 16 -> bitmap_offset = 64
        let hdr = ForceQueueHeader {
            capacity: 1024,
            head: 0,
            tail: 0,
            generation: 0,
            stride: 7,
            flags: 0,
        };
        assert_eq!(hdr.bitmap_words(), 16);
        assert_eq!(hdr.bitmap_offset(), 64);
        assert_eq!(hdr.payload_offset(), 64 + 16 * 8);
        assert_eq!(hdr.total_size(), 64 + 16 * 8 + 1024 * 7 * 8);
    }

    #[test]
    fn force_queue_header_validate() {
        // Valid: power-of-2 capacity, valid stride
        let hdr = ForceQueueHeader {
            capacity: 1024,
            head: 0,
            tail: 0,
            generation: 0,
            stride: 6,
            flags: 0,
        };
        assert_eq!(hdr.validate(), 0);

        let hdr = ForceQueueHeader {
            capacity: 1024,
            head: 0,
            tail: 0,
            generation: 0,
            stride: 7,
            flags: 0,
        };
        assert_eq!(hdr.validate(), 0);

        // Invalid: non-power-of-2 capacity
        let hdr = ForceQueueHeader {
            capacity: 1000,
            head: 0,
            tail: 0,
            generation: 0,
            stride: 7,
            flags: 0,
        };
        assert_eq!(hdr.validate(), 2);

        // Invalid: zero capacity
        let hdr = ForceQueueHeader {
            capacity: 0,
            head: 0,
            tail: 0,
            generation: 0,
            stride: 7,
            flags: 0,
        };
        assert_eq!(hdr.validate(), 2);

        // Invalid: stride not 6 or 7
        let hdr = ForceQueueHeader {
            capacity: 1024,
            head: 0,
            tail: 0,
            generation: 0,
            stride: 5,
            flags: 0,
        };
        assert_eq!(hdr.validate(), 2);

        let hdr = ForceQueueHeader {
            capacity: 1024,
            head: 0,
            tail: 0,
            generation: 0,
            stride: 8,
            flags: 0,
        };
        assert_eq!(hdr.validate(), 2);
    }
}
