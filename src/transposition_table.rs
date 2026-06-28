// Transposition Table implementation.
// https://www.chessprogramming.org/Transposition_Table

use mouse::backend::constants::A1;
use mouse::moove::Moove;
use crate::engine::{MATE_SCORE, MAX_DEPTH};

const SIZE_OF_ENTRY_BYTES: usize = size_of::<TTEntry>();
const BUCKET_LENGTH: usize = 2;

#[derive(Clone, Eq, PartialEq, Debug, Copy)]
pub enum TTEntryType {
    Exact,
    UpperBound,
    LowerBound,
}

#[derive(Clone)]
pub struct TTEntry {
    pub zobrist_hash: u64,
    pub recommended_move: Moove,
    pub depth: u8,
    pub score: i32,
    pub entry_type: TTEntryType,
    pub empty: bool,
    pub generation: u16,
}

impl TTEntry {
    pub fn new(
        zobrist_hash: u64,
        recommended_move: Moove,
        depth: u8,
        score: i32,
        entry_type: TTEntryType,
        generation: u16,
    ) -> Self {
        Self {
            zobrist_hash,
            recommended_move,
            depth,
            score,
            entry_type,
            empty: false,
            generation,
        }
    }
}

impl Default for TTEntry {
    fn default() -> Self {
        Self {
            zobrist_hash: 0,
            recommended_move: Moove::new(A1, A1),
            depth: 0,
            score: 0,
            entry_type: TTEntryType::Exact,
            empty: true,
            generation: 0,
        }
    }
}

#[cfg(feature = "dev")]
struct TTLogging {
    // The following variables are only used for some logging :)
    num_entries: usize,
    collisions: usize,
    get_attempts: usize,
    get_hits: usize,
}

// Currently the TT follows a bucket design with 2 entries where the first entry prioritizes deep searches and the second one always get replaced.

// Potential TT Table improvements
// 1. try it for q-search as well.
// 2. try not to store fail high cases.
// 3. 16 bit keys
// 4. Bitmap the entries to compress them.
pub struct TranspositionTable {
    tt: Vec<[TTEntry; BUCKET_LENGTH]>,
    generation: u16,
    #[cfg(feature = "dev")]
    log: TTLogging,
}

impl TranspositionTable {
    // --- Constructors and Basics --- //
    pub fn new() -> Self {
        // Default to a size of 64MB cause why not.
        Self::new_with_mb(64)
    }

    pub fn new_with_mb(mb: usize) -> Self {
        let desired_len = (mb * 1000 * 1000) / (SIZE_OF_ENTRY_BYTES * BUCKET_LENGTH);

        let tt = vec![[TTEntry::default(), TTEntry::default()]; desired_len];
        println!(
            "info string tt-len {}, tt-MB {} entry-B {}",
            tt.len(),
            (tt.len() * BUCKET_LENGTH * SIZE_OF_ENTRY_BYTES) / 1000 / 1000,
            SIZE_OF_ENTRY_BYTES
        );
        let generation = 0;
        #[cfg(feature = "dev")]
        {
           let log = TTLogging {
                num_entries: 0,
                collisions: 0,
                get_attempts: 0,
                get_hits: 0,
            };
            Self { tt, generation, log }
        }
        #[cfg(not(feature = "dev"))]
        Self { tt, generation }
    }
    
    pub fn next_generation(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }

    fn get_index(&self, zobrist_hash: u64) -> usize {
        zobrist_hash as usize % self.tt.len()
    }
    
    fn get_entry(&mut self, zobrist_hash: u64) -> Option<TTEntry> {
        let bucket = &self.tt[self.get_index(zobrist_hash)];

        for index in 0..BUCKET_LENGTH {
            if bucket[index].empty || bucket[index].zobrist_hash != zobrist_hash {
                continue;
            }
            return Some(bucket[index].clone())
        }

        None       
    }
    
    #[cfg(feature = "dev")]
    pub fn print_info_string(&self) {
        println!(
            "info string fullness {:.2}%, get-hit {:.2}%, collisions {}",
            (self.log.num_entries as f32 / (self.tt.len() * BUCKET_LENGTH) as f32) * 100.0,
            (self.log.get_hits as f32 / self.log.get_attempts as f32) * 100.0,
            self.log.collisions,
        )
    }


    // --- Core Features --- //

    pub fn get_hash_move(&mut self, zobrist_hash: u64) -> Option<Moove> {
        Some(self.get_entry(zobrist_hash)?.recommended_move)
    }

    pub fn can_tt_prune(
        &mut self,
        zobrist_hash: u64,
        ply: u8,
        remaining_depth: u8,
        alpha: i32,
        beta: i32,
        is_pv: bool,
    ) -> Option<i32> {
        // https://www.chessprogramming.org/Transposition_Table#Using_the_Transposition_Table
        // A cutoff can be performed when the depth of entry is greater (or equal) to the depth of the current node
        // and one of the following criteria is satisfied:
        //
        //     The entry type is EXACT
        //     The entry type is LOWER BOUND and greater than or equal to beta
        //     The entry type is UPPER BOUND and less than alpha
        #[cfg(feature = "dev")] { self.log.get_attempts += 1; }
        let mut tt_entry = self.get_entry(zobrist_hash)?;
        #[cfg(feature = "dev")] { self.log.get_hits += 1; }

        // Adjust for mate distance!
        // Is this a mate?
        if tt_entry.score > MATE_SCORE - MAX_DEPTH as i32 {
            // winning mate
            tt_entry.score -= ply as i32
        } else if tt_entry.score < -MATE_SCORE + MAX_DEPTH as i32 {
            // losing mate
            tt_entry.score += ply as i32
        }

        let tt_score = tt_entry.score;

        // If the current depth is greater than the depth of the entry, we can't prune.
        if remaining_depth > tt_entry.depth {
            return None;
        }

        match tt_entry.entry_type {
            // If the entry type is EXACT, we can prune.
            // The entry is only exact if the node is a PV-node.
            TTEntryType::Exact => return Some(tt_score),

            // The entry is an Upper Bound if everything was searched, but nothing improved alpha.
            TTEntryType::UpperBound => {
                // The real score of this position is max(tt_entry.score)
                // and if even this maximum is worse than alpha, we can prune.
                if !is_pv && tt_entry.score <= alpha {
                    return Some(tt_score);
                }
            }

            // The entry is a Lower Bound if it lead to pruning.
            TTEntryType::LowerBound => {
                // Similarly, the real score of this position is min(tt_entry.score)
                // and if even this minimum is better than beta, we can prune
                // because the opposing player won't let us reach this position.
                if !is_pv && tt_entry.score >= beta {
                    return Some(tt_score);
                }
            }
        }

        None
    }

    pub fn update_tt_table(
        &mut self,
        zobrist_hash: u64,
        mut max_score: i32,
        best_node_move: Moove,
        remaining_depth: u8,
        node_type: TTEntryType,
        ply: u8,
    ) {
        // Is this a mate?
        if max_score > MATE_SCORE - MAX_DEPTH as i32{
            // winning mate
            max_score += ply as i32;
        } else if max_score < -MATE_SCORE + MAX_DEPTH as i32{
            // losing mate
            max_score -= ply as i32;
        }

        let entry = TTEntry::new(
            zobrist_hash,
            best_node_move,
            remaining_depth,
            max_score,
            node_type,
            self.generation
        );
        let bucket_index = self.get_index(entry.zobrist_hash);
        let bucket = &mut self.tt[bucket_index];

        // --- Case 1: One of the entries is empty. --- //
        for index in 0..BUCKET_LENGTH {
            if bucket[index].empty {
                #[cfg(feature = "dev")] { self.log.num_entries +=1; }
                bucket[index] = entry;
                return;
            }
        }

        #[cfg(feature = "dev")] { self.log.collisions +=1; }
        // --- Case 2: One of the entries is for the current position. --- //
        for index in 0..BUCKET_LENGTH {
            // In that case just replace it.
            if bucket[index].zobrist_hash == zobrist_hash {
                bucket[index] = entry;
                return;
            }
        }

        // --- Case 3: Decide what to replace based on a score. --- //
        let mut worst_index = 0;
        let mut worst_score = i64::MAX;

        for index in 0..BUCKET_LENGTH {
            let cur_entry = &bucket[index];
            let age = (self.generation - cur_entry.generation) as i64;
            let score = cur_entry.depth as i64 - age; // Higher = better

            if score < worst_score {
                worst_score = score;
                worst_index = index;
            }
        }

        // Replace the worst entry (lowest depth, oldest)
        bucket[worst_index] = entry;
    }
}
