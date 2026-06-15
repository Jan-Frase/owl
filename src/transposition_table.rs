// Transposition Table implementation.
// https://www.chessprogramming.org/Transposition_Table

use mouse::backend::constants::A1;
use mouse::moove::Moove;

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

pub struct TranspositionTable {
    tt: Vec<TTEntry>,
}

// Basic operations
impl TranspositionTable {
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
        Self { tt }
    }

    fn get_index(&self, zobrist_hash: u64) -> usize {
        zobrist_hash as usize % self.tt.len()
    }

    pub fn add_entry(&mut self, entry: TTEntry) {
        let index = self.get_index(entry.zobrist_hash);
        let old_entry = &self.tt[index];

        // If there is no previous entry,
        if old_entry.empty {
            // simply insert the entry.
            self.tt[index] = entry;
            return;
        }

        // Otherwise, check if it's worth replacing.
        // If the old entry is deeper, don't replace it.
        if old_entry.depth > entry.depth {
            return;
        }
        // Otherwise, just overwrite it.
        self.tt[index] = entry;
    }

    pub fn get_entry(&self, zobrist_hash: u64) -> Option<&TTEntry> {
        let index = self.get_index(zobrist_hash);
        let entry = &self.tt[index];

        if entry.empty {
            return None;
        }
        if entry.zobrist_hash != zobrist_hash {
            return None;
        }
        Some(entry)
    }
}

impl TranspositionTable {
    pub fn can_tt_prune(
        &self,
        tt_entry: Option<&TTEntry>,
        current_depth: u8,
        alpha: i32,
        beta: i32,
    ) -> bool {
        // https://www.chessprogramming.org/Transposition_Table#Using_the_Transposition_Table
        // A cutoff can be performed when the depth of entry is greater (or equal) to the depth of the current node
        // and one of the following criteria is satisfied:
        //
        //     The entry type is EXACT
        //     The entry type is LOWER BOUND and greater than or equal to beta
        //     The entry type is UPPER BOUND and less than alpha

        // If we have no entry, we can't prune.
        let tt_entry = match tt_entry {
            None => return false,
            Some(tt_entry) => tt_entry,
        };

        // If the current depth is greater than the depth of the entry, we can't prune.
        if current_depth > tt_entry.depth {
            return false;
        }

        match tt_entry.entry_type {
            // If the entry type is EXACT, we can prune.
            // The entry is only exact if the node is a PV-node.
            TTEntryType::Exact => return true,

            // The entry is an Upper Bound if everything was searched, but nothing improved alpha.
            TTEntryType::UpperBound => {
                // The real score of this position is max(tt_entry.score)
                // and if even this maximum is worse than alpha, we can prune.
                if tt_entry.score <= alpha {
                    return true;
                }
            }

            // The entry is a Lower Bound if it lead to pruning.
            TTEntryType::LowerBound => {
                // Similarly, the real score of this position is min(tt_entry.score)
                // and if even this minimum is better than beta, we can prune
                // because the opposing player won't let us reach this position.
                if tt_entry.score >= beta {
                    return true;
                }
            }
        }

        false
    }

    pub fn update_tt_table(
        &mut self,
        zobrist_hash: u64,
        max_score: i32,
        best_node_move: Moove,
        current_depth: u8,
        original_alpha: i32,
        beta: i32,
        mut node_type: TTEntryType,
    ) {
        if max_score <= original_alpha {
            // We have searched all moves but found no improvement on alpha.
            node_type = TTEntryType::UpperBound;
        } else if max_score < beta {
            node_type = TTEntryType::Exact;
        }

        let entry = TTEntry::new(
            zobrist_hash,
            best_node_move,
            current_depth,
            max_score,
            node_type,
        );

        self.add_entry(entry);
    }
}
