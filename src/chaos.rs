use rand::Rng;

/// Allocates a massive chunk of memory filled with random high-entropy data.
/// Purpose:
/// 1. Exhaust memory in lightweight analysis VMs (often < 4GB).
/// 2. Pollute memory dumps, making string search/extraction harder.
/// 3. Confuse entropy analysis tools.
/// 
/// Returns a handle directly to the ocean. The caller must keep it alive.
pub struct EntropyOcean {
    _data: Vec<u8>,
}

impl EntropyOcean {
    pub fn summon() -> Self {
        // Target: 256 MB
        const OCEAN_SIZE: usize = 256 * 1024 * 1024;
        
        // We use Vec::with_capacity to request the memory from OS.
        let mut ocean = Vec::with_capacity(OCEAN_SIZE);
        
        // We must actually write to it to force the OS to commit the pages.
        // Just allocating might be lazy.
        // Also, we want high entropy.
        
        // Optimization: Generating 256MB of cryptographically secure random numbers is slow.
        // We use a fast non-cryptographic fill or a repeating pattern of garbage.
        // Let's use a fast XORShift or similar inline for speed, or just `rand::thread_rng().fill()`.
        // `rand` might be slow for 256MB on a single thread startup.
        
        // Faster strategy: Allocate, verify capacity, write random noise to valid pages.
        // Let's fill it with "Semi-Chaos": repeating blocks of random data.
        
        let mut rng = rand::thread_rng();
        let chunk_size = 1024 * 64; // 64KB chunk
        let mut noise_chunk = vec![0u8; chunk_size];
        rng.fill(&mut noise_chunk[..]);

        // unsafe set_len? No, let's just extend.
        // ocean.extend_from_slice... loop
        
        // 256MB / 64KB = 4096 chunks
        for _ in 0..4096 {
            ocean.extend_from_slice(&noise_chunk);
            // Mutate the chunk slightly so it's not identical (deduplication defeat)
            noise_chunk[0] = noise_chunk[0].wrapping_add(1);
        }

        EntropyOcean { _data: ocean }
    }
}
