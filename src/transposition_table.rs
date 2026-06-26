// Transposition Table implementation.
// https://www.chessprogramming.org/Transposition_Table

use mouse::backend::constants::A1;
use mouse::moove::Moove;
use crate::engine::{MATE_SCORE, MAX_DEPTH};

const SIZE_OF_ENTRY_BYTES: usize = size_of::<TTEntry>();
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
}

impl TTEntry {
    pub fn new(
        zobrist_hash: u64,
        recommended_move: Moove,
        depth: u8,
        score: i32,
        entry_type: TTEntryType,
    ) -> Self {
        Self {
            zobrist_hash,
            recommended_move,
            depth,
            score,
            entry_type,
            empty: false,
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
        }
    }
}

#[cfg(feature = "dev")]
struct TTLogging {
    // The following variables are only used for some logging :)
    num_entries: usize,
    collisions: usize,
    overwrites: usize,

    get_attempts: usize,
    get_hits: usize,
}

// Potential TT Table improvements
// 4. try to have multiple buckets at each slot
// 1. try it for q-search as well.
// 2. try not to store fail high cases.
// 3. 16 bit keys
pub struct TranspositionTable {
    tt: Vec<TTEntry>,
    #[cfg(feature = "dev")]
    log: TTLogging,
}

impl TranspositionTable {
    // --- Constructors --- //
    pub fn new() -> Self {
        // Default to a size of 64MB cause why not.
        Self::new_with_mb(64)
    }

    pub fn new_with_mb(mb: usize) -> Self {
        let desired_len = (mb * 1000 * 1000) / SIZE_OF_ENTRY_BYTES;

        let tt = vec![TTEntry::default(); desired_len];
        println!(
            "info string tt-len {}, tt-MB {} entry-B {}",
            tt.len(),
            (tt.len() * SIZE_OF_ENTRY_BYTES) / 1000 / 1000,
            SIZE_OF_ENTRY_BYTES
        );
        #[cfg(feature = "dev")]
        {
           let log = TTLogging {
                num_entries: 0,
                collisions: 0,
                overwrites: 0,
                get_attempts: 0,
                get_hits: 0,
            };
            Self { tt, log }
        }
        #[cfg(not(feature = "dev"))]
        Self { tt }
    }

    // --- Quick Helpers --- //
    #[cfg(feature = "dev")]
    pub fn print_info_string(&self) {
        println!(
            "info string fullness {:.2}%, collision {}, overwrites {}, get-hit {:.2}%",
            (self.log.num_entries as f32 / self.tt.len() as f32) * 100.0,
            self.log.collisions,
            self.log.overwrites,
            (self.log.get_hits as f32 / self.log.get_attempts as f32) * 100.0,
        )
    }

    fn get_index(&self, zobrist_hash: u64) -> usize {
        zobrist_hash as usize % self.tt.len()
    }

    pub fn get_hash_move(&self, zobrist_hash: u64) -> Option<Moove> {
        let index = self.get_index(zobrist_hash);
        let entry = &self.tt[index];

        if entry.empty {
            return None;
        }
        if entry.zobrist_hash != zobrist_hash {
            return None;
        }
        Some(entry.recommended_move)
    }

    fn get_entry(&mut self, zobrist_hash: u64, ply: u8) -> Option<TTEntry> {
        #[cfg(feature = "dev")]
        {
            self.log.get_attempts += 1;
        }
        let index = self.get_index(zobrist_hash);
        let mut entry = self.tt[index].clone();

        if entry.empty {
            return None;
        }
        if entry.zobrist_hash != zobrist_hash {
            return None;
        }

        // Is this a mate?
        if entry.score > MATE_SCORE - MAX_DEPTH as i32 {
            // winning mate
            entry.score -= ply as i32
        } else if entry.score < -MATE_SCORE + MAX_DEPTH as i32 {
            // losing mate
            entry.score += ply as i32
        }

        #[cfg(feature = "dev")]
        {
            self.log.get_hits += 1;
        }
        Some(entry)
    }

    // --- Search Specific --- //
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
        let tt_entry = self.get_entry(zobrist_hash, ply);

        // If we have no entry, we can't prune.
        let tt_entry = match tt_entry {
            None => return None,
            Some(tt_entry) => tt_entry,
        };

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
        );

        let index = self.get_index(entry.zobrist_hash);
        let old_entry = &self.tt[index];

        // If there is no previous entry,
        if old_entry.empty {
            #[cfg(feature = "dev")]
            {
                self.log.num_entries += 1;
            }
            // simply insert the entry.
            self.tt[index] = entry;
            return;
        }

        // Otherwise, check if it's worth replacing.
        // If the old entry is deeper, don't replace it.
        #[cfg(feature = "dev")]
        {
            self.log.collisions += 1;
        }
        if old_entry.zobrist_hash == entry.zobrist_hash && old_entry.depth > entry.depth {
            return;
        }
        // Otherwise, just overwrite it.
        #[cfg(feature = "dev")]
        {
            self.log.overwrites += 1;
        }
        self.tt[index] = entry;
    }
}
