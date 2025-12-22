use std::collections::VecDeque;

/// A single encoded media packet with a presentation timestamp.
/// `pts_ms` must be monotonically increasing.
#[derive(Debug, Clone)]
pub struct Packet {
    pub pts_ms: u64,
    pub data: Vec<u8>,
}

pub struct RingBuffer {
    max_duration_ms: u64,
    packets: VecDeque<Packet>,
}

impl RingBuffer {
    pub fn new(max_duration_ms: u64) -> Self {
        Self {
            max_duration_ms,
            packets: VecDeque::new(),
        }
    }

    pub fn push(&mut self, packet: Packet) {
        self.packets.push_back(packet);
        self.evict_old_packets();
    }

    fn evict_old_packets(&mut self) {
        let Some(newest) = self.packets.back() else {
            return;
        };

        let newest_pts = newest.pts_ms;

        while let Some(oldest) = self.packets.front() {
            if newest_pts.saturating_sub(oldest.pts_ms) > self.max_duration_ms {
                self.packets.pop_front();
            } else {
                break;
            }
        }
    }

    // Return a snapshot of the current packets in the buffer
    pub fn snapshot(&self) -> Vec<Packet> {
        self.packets.iter().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.packets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.packets.is_empty()
    }

    pub fn duration_ms(&self) -> u64 {
        match (self.packets.front(), self.packets.back()) {
            (Some(first), Some(last)) => last.pts_ms.saturating_sub(first.pts_ms),
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn packet(pts_ms: u64) -> Packet {
        Packet {
            pts_ms,
            data: vec![0; 10],
        }
    }

    #[test]
    fn pushes_and_keeps_packets_within_duration() {
        let mut buffer = RingBuffer::new(3000);

        buffer.push(packet(0));
        buffer.push(packet(1000));
        buffer.push(packet(2000));
        buffer.push(packet(3000));

        assert_eq!(buffer.len(), 4);
        assert_eq!(buffer.duration_ms(), 3000);
    }

    #[test]
    fn evicts_packets_outside_duration() {
        let mut buffer = RingBuffer::new(2000);

        buffer.push(packet(0));
        buffer.push(packet(1000));
        buffer.push(packet(2000));
        buffer.push(packet(3000));

        // 3000 - 0 = 3000 > 2000 → evict
        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.snapshot()[0].pts_ms, 1000);
    }

    #[test]
    fn snapshot_preserves_order() {
        let mut buffer = RingBuffer::new(5000);

        buffer.push(packet(10));
        buffer.push(packet(20));
        buffer.push(packet(30));

        let snapshot = buffer.snapshot();

        assert_eq!(snapshot[0].pts_ms, 10);
        assert_eq!(snapshot[1].pts_ms, 20);
        assert_eq!(snapshot[2].pts_ms, 30);
    }
}
