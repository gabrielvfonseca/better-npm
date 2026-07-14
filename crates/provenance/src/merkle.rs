use serde::{Deserialize, Serialize};

const LEAF_PREFIX: u8 = 0x00;
const NODE_PREFIX: u8 = 0x01;

fn leaf_hash(data: &[u8; 32]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&[LEAF_PREFIX]);
    hasher.update(data);
    *hasher.finalize().as_bytes()
}

fn internal_hash(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&[NODE_PREFIX]);
    hasher.update(left);
    hasher.update(right);
    *hasher.finalize().as_bytes()
}

fn largest_power_of_2(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    let mut k = 1;
    while k < n {
        k <<= 1;
    }
    k >> 1
}

fn subtree_hash(entries: &[[u8; 32]], count: u64) -> [u8; 32] {
    if count == 0 {
        return [0u8; 32];
    }
    if count == 1 {
        return leaf_hash(&entries[0]);
    }
    let k = largest_power_of_2(count);
    let left = subtree_hash(entries, k);
    let right = subtree_hash(&entries[k as usize..], count - k);
    internal_hash(&left, &right)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InclusionProof {
    pub leaf_index: u64,
    pub tree_size: u64,
    pub siblings: Vec<[u8; 32]>,
}

#[derive(Debug, Clone, Default)]
pub struct MerkleTree {
    entries: Vec<[u8; 32]>,
}

impl MerkleTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append(&mut self, hash: [u8; 32]) -> u64 {
        let index = self.entries.len() as u64;
        self.entries.push(hash);
        index
    }

    fn path_in(&self, m: u64, n: u64, offset: usize) -> Vec<[u8; 32]> {
        if n == 1 {
            return Vec::new();
        }
        let k = largest_power_of_2(n);
        if m < k {
            let mut sub = self.path_in(m, k, offset);
            sub.push(subtree_hash(&self.entries[(offset + k as usize)..], n - k));
            sub
        } else {
            let mut sub = self.path_in(m - k, n - k, offset + k as usize);
            sub.push(subtree_hash(&self.entries[offset..(offset + k as usize)], k));
            sub
        }
    }

    pub fn inclusion_proof(&self, leaf_index: u64) -> InclusionProof {
        let tree_size = self.entries.len() as u64;
        let siblings = self.path_in(leaf_index, tree_size, 0);
        InclusionProof { leaf_index, tree_size, siblings }
    }

    pub fn root_hash(&self) -> [u8; 32] {
        if self.entries.is_empty() {
            return [0u8; 32];
        }
        subtree_hash(&self.entries, self.entries.len() as u64)
    }

    pub fn size(&self) -> u64 {
        self.entries.len() as u64
    }
}

pub fn verify_inclusion(
    proof: &InclusionProof,
    entry_hash: &[u8; 32],
    expected_root: &[u8; 32],
) -> bool {
    if proof.leaf_index >= proof.tree_size {
        return false;
    }

    let mut fn_idx = proof.leaf_index;
    let mut sn = proof.tree_size - 1;
    let mut r = leaf_hash(entry_hash);

    for sibling in &proof.siblings {
        if fn_idx == sn || fn_idx % 2 == 1 {
            r = internal_hash(sibling, &r);
            while fn_idx % 2 == 0 && fn_idx != 0 {
                fn_idx /= 2;
                sn /= 2;
            }
            fn_idx /= 2;
            sn /= 2;
        } else {
            r = internal_hash(&r, sibling);
            fn_idx /= 2;
            sn /= 2;
        }
    }

    r == *expected_root
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_hash(i: u8) -> [u8; 32] {
        let mut h = [0u8; 32];
        h[0] = i;
        h
    }

    #[test]
    fn test_empty_tree() {
        let tree = MerkleTree::new();
        assert_eq!(tree.size(), 0);
        assert_eq!(tree.root_hash(), [0u8; 32]);
    }

    #[test]
    fn test_single_entry() {
        let mut tree = MerkleTree::new();
        let idx = tree.append(test_hash(1));
        assert_eq!(idx, 0);
        assert_eq!(tree.size(), 1);
        let root = tree.root_hash();
        assert_ne!(root, [0u8; 32]);
    }

    #[test]
    fn test_deterministic_root() {
        let mut t1 = MerkleTree::new();
        let mut t2 = MerkleTree::new();
        for i in 0..10 {
            t1.append(test_hash(i));
            t2.append(test_hash(i));
        }
        assert_eq!(t1.root_hash(), t2.root_hash());
        assert_eq!(t1.size(), 10);
        assert_eq!(t2.size(), 10);
    }

    #[test]
    fn test_inclusion_proof_verify() {
        let mut tree = MerkleTree::new();
        let hashes: Vec<[u8; 32]> = (0..10).map(test_hash).collect();
        let mut indices = Vec::new();
        for h in &hashes {
            indices.push(tree.append(*h));
        }
        let root = tree.root_hash();

        for (i, h) in hashes.iter().enumerate() {
            let proof = tree.inclusion_proof(i as u64);
            assert!(verify_inclusion(&proof, h, &root));
        }
    }

    #[test]
    fn test_inclusion_proof_tampered_entry() {
        let mut tree = MerkleTree::new();
        let hashes: Vec<[u8; 32]> = (0..10).map(test_hash).collect();
        for h in &hashes {
            tree.append(*h);
        }
        let root = tree.root_hash();

        let proof = tree.inclusion_proof(5);
        let tampered = test_hash(0xff);
        assert!(!verify_inclusion(&proof, &tampered, &root));
    }

    #[test]
    fn test_inclusion_proof_tampered_root() {
        let mut tree = MerkleTree::new();
        let hashes: Vec<[u8; 32]> = (0..10).map(test_hash).collect();
        for h in &hashes {
            tree.append(*h);
        }
        let root = tree.root_hash();

        let proof = tree.inclusion_proof(5);
        let mut bad_root = root;
        bad_root[0] ^= 0xff;
        assert!(!verify_inclusion(&proof, &hashes[5], &bad_root));
    }

    #[test]
    fn test_inclusion_proof_out_of_bounds() {
        let mut tree = MerkleTree::new();
        for i in 0..5 {
            tree.append(test_hash(i));
        }
        let root = tree.root_hash();
        let proof = tree.inclusion_proof(2);
        assert!(!verify_inclusion(&proof, &test_hash(2), &[0u8; 32]));
        let empty_proof =
            InclusionProof { leaf_index: 100, tree_size: tree.size(), siblings: vec![] };
        assert!(!verify_inclusion(&empty_proof, &test_hash(0), &root));
    }

    #[test]
    fn test_power_of_two_sizes() {
        for size in &[1u64, 2, 4, 8, 16] {
            let mut tree = MerkleTree::new();
            let hashes: Vec<[u8; 32]> = (0..*size)
                .map(|i| {
                    let mut h = [0u8; 32];
                    h[0] = i as u8;
                    h
                })
                .collect();
            for h in &hashes {
                tree.append(*h);
            }
            let root = tree.root_hash();
            for i in 0..*size {
                let proof = tree.inclusion_proof(i);
                assert!(
                    verify_inclusion(&proof, &hashes[i as usize], &root),
                    "inclusion proof failed at size={size} index={i}"
                );
            }
        }
    }

    #[test]
    fn test_odd_sizes() {
        for size in &[3u64, 5, 7, 9, 11] {
            let mut tree = MerkleTree::new();
            let hashes: Vec<[u8; 32]> = (0..*size)
                .map(|i| {
                    let mut h = [0u8; 32];
                    h[0] = i as u8;
                    h
                })
                .collect();
            for h in &hashes {
                tree.append(*h);
            }
            let root = tree.root_hash();
            for i in 0..*size {
                let proof = tree.inclusion_proof(i);
                assert!(
                    verify_inclusion(&proof, &hashes[i as usize], &root),
                    "inclusion proof failed at size={size} index={i}"
                );
            }
        }
    }
}
