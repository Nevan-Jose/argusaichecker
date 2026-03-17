pub mod ranker;

use crate::schemas::violations::ViolationCluster;
use crate::schemas::review::ReviewCandidate;

pub fn rank(clusters: &[ViolationCluster]) -> Vec<ReviewCandidate> {
    ranker::rank_clusters(clusters)
}
