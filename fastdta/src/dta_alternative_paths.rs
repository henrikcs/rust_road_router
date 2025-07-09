use std::path::Path;

use conversion::{
    FILE_DTA_QUERIES_ALTERNATIVE_CHOICE, FILE_DTA_QUERIES_ALTERNATIVE_COST, FILE_DTA_QUERIES_ALTNERNATIVE_PROBABILITIES, FILE_DTA_QUERIES_EDGE_IDS,
    FILE_DTA_QUERIES_FIRST_ALTERNATIVE_PATH, FILE_DTA_QUERIES_FIRST_EDGE_OF_ALTERNATIVE,
};
use rand::{
    SeedableRng,
    distr::{Distribution, weighted::WeightedIndex},
    rngs::StdRng,
};
use rust_road_router::{
    datastr::graph::EdgeId,
    io::{Load, Store},
};

use crate::choice::{ChoiceAlgorithm, gawron, logit};

#[derive(Debug, Clone)]
pub struct AlternativePathsForDTA {
    /// queries i has alternatives alternatives[i]
    pub alternatives: Vec<AlternativePaths>,
}

/// paths, costs, probabilities and choice for one query
/// each vector has the same length, which is the number of alternatives for the query
/// choice is a number in [0, paths.len() - 1]
#[derive(Debug, Clone)]
pub struct AlternativePaths {
    pub paths: Vec<AlternativePath>,
    pub costs: Vec<f64>,
    pub probabilities: Vec<f64>,
    pub choice: usize,
}

impl AlternativePaths {
    pub fn perform_choice_model(&mut self, choice_algorithm: &ChoiceAlgorithm, max_alternatives: u32) {
        self.set_probabilities(&choice_algorithm);

        // check if we have more alternatives than allowed
        if self.paths.len() > max_alternatives as usize {
            // remove the least probable choices and rescale probabilities
            let mut prob_with_indices: Vec<(f64, usize)> = self.probabilities.iter().enumerate().map(|(index, &p)| (p, index)).collect();
            prob_with_indices.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

            // get the indices to remove
            let indices_to_remove: Vec<usize> = prob_with_indices
                .iter()
                .take(prob_with_indices.len() - max_alternatives as usize)
                .map(|&(_, index)| index)
                .collect();

            // filter out the indices to remove
            self.paths = self
                .paths
                .iter()
                .enumerate()
                .filter(|(index, _)| !indices_to_remove.contains(index))
                .map(|(_, w)| w.clone())
                .collect();

            self.costs = self
                .costs
                .iter()
                .enumerate()
                .filter(|(index, _)| !indices_to_remove.contains(index))
                .map(|(_, w)| *w)
                .collect();

            // rescale probabilities
            self.set_probabilities(&choice_algorithm);
        }
    }

    pub fn choose(&mut self, rng: &mut StdRng) {
        if self.paths.is_empty() {
            return;
        }

        let prob = WeightedIndex::new(&self.probabilities).expect("Failed to create weighted index");
        self.choice = prob.sample(rng);
    }

    pub fn set_probabilities(&mut self, choice_algorithm: &ChoiceAlgorithm) {
        match choice_algorithm {
            ChoiceAlgorithm::Gawron { a, beta } => {
                self.probabilities = gawron(&self.paths.iter().map(|p| p.edges.clone()).collect(), &self.costs, *a, *beta);
            }
            ChoiceAlgorithm::Logit { theta } => {
                self.probabilities = logit(&self.costs, *theta);
            }
        }
    }
}

/// path represented by a sequence of edge id used in TDGraph
#[derive(Debug, Clone)]
pub struct AlternativePath {
    pub edges: Vec<EdgeId>,
}

impl AlternativePathsForDTA {
    pub fn perform_choice_model(&mut self, _previous_alternatives: &Self, choice_algorithm: &ChoiceAlgorithm, max_alternatives: u32, seed: i32) {
        let mut rng: StdRng = StdRng::seed_from_u64(seed.abs() as u64);

        for alternative_paths in self.alternatives.iter_mut() {
            alternative_paths.perform_choice_model(choice_algorithm, max_alternatives);

            alternative_paths.choose(&mut rng);
        }
    }
}

impl AlternativePathsForDTAFlattened {
    pub fn reconstruct(dir: &Path) -> Self {
        // read from given directory the files with the same name as the fields of this struct
        // we can use Vec<>::load_from() for easier reading

        let edges: Vec<u32> = Vec::<u32>::load_from(dir.join(FILE_DTA_QUERIES_EDGE_IDS)).unwrap();
        let first_alternative_of_query: Vec<u32> = Vec::<u32>::load_from(dir.join(FILE_DTA_QUERIES_FIRST_ALTERNATIVE_PATH)).unwrap();
        let first_edge_of_alternative: Vec<u32> = Vec::<u32>::load_from(dir.join(FILE_DTA_QUERIES_FIRST_EDGE_OF_ALTERNATIVE)).unwrap();
        let alternative_costs: Vec<f64> = Vec::<f64>::load_from(dir.join(FILE_DTA_QUERIES_ALTERNATIVE_COST)).unwrap();
        let alternative_choice: Vec<u32> = Vec::<u32>::load_from(dir.join(FILE_DTA_QUERIES_ALTERNATIVE_CHOICE)).unwrap();
        let alternative_probabilities: Vec<f64> = Vec::<f64>::load_from(dir.join(FILE_DTA_QUERIES_ALTNERNATIVE_PROBABILITIES)).unwrap();

        assert_eq!(
            alternative_choice.len(),
            first_alternative_of_query.len() - 1,
            "first_alternative_of_query has to be of size q + 1, where q is the number of queries."
        );

        assert_eq!(
            first_edge_of_alternative.len() - 1,
            alternative_probabilities.len(),
            "First edge of alternative must have length equal to the number of alternatives + 1 (which is altnative_probabilities.len() + 1)"
        );

        assert_eq!(
            alternative_costs.len(),
            alternative_probabilities.len(),
            "alternative_costs and alternative_probabilities must have the same length"
        );

        Self {
            egdes: edges,
            first_alternative_of_query,
            first_edge_of_alternative,
            alternative_costs,
            alternative_choice,
            alternative_probabilities,
        }
    }

    pub fn deconstruct(&self, dir: &Path) -> Result<(), std::io::Error> {
        // write the fields to files in the given directory
        self.egdes.write_to(&dir.join(FILE_DTA_QUERIES_EDGE_IDS))?;
        self.first_alternative_of_query.write_to(&dir.join(FILE_DTA_QUERIES_FIRST_ALTERNATIVE_PATH))?;
        self.first_edge_of_alternative.write_to(&dir.join(FILE_DTA_QUERIES_FIRST_EDGE_OF_ALTERNATIVE))?;
        self.alternative_costs.write_to(&dir.join(FILE_DTA_QUERIES_ALTERNATIVE_COST))?;
        self.alternative_choice.write_to(&dir.join(FILE_DTA_QUERIES_ALTERNATIVE_CHOICE))?;
        self.alternative_probabilities
            .write_to(&dir.join(FILE_DTA_QUERIES_ALTNERNATIVE_PROBABILITIES))?;

        Ok(())
    }
}

/// datastructure to hold alternative paths and their properties
///
/// This structure is used to store the alternative paths for each query,
/// including the edges of paths, costs of alternative, and probabilities to choose an alternative and the one which got chosen.
///
/// Let the number of queries be q.
///
#[derive(Debug, Clone)]
struct AlternativePathsForDTAFlattened {
    /// contains all edges of all alternative paths of queries used during DTA, where edges are encoded as u32 indices
    pub egdes: Vec<u32>,
    /// contains the index of the first alternative to a set of alternative paths.
    /// Query with index i has its first alternative path starting at edge_ids[first_edge_of_alternative[first_alternative_of_query[i]]]
    /// Number of alternatives for query i is given by first_alternative_of_query[i + 1] - first_alternative_of_query[i]
    pub first_alternative_of_query: Vec<u32>,
    /// Contains the index of the first edge of the alternative path.
    /// length of a path is given by first_edge_of_alternative[i + 1] - first_edge_of_alternative[i]
    pub first_edge_of_alternative: Vec<u32>,
    /// contains the stochastic cost of an alternative path
    /// the cost of the i-th alternative path of query j is given by
    /// alternative_costs[first_alternative_of_query[j] + i]
    pub alternative_costs: Vec<f64>,
    /// the choice of the alternative path, i.e. the index of the alternative path in the alternative_paths vector
    /// /// contains information which alternative path was chosen for each query
    /// if i-th alternative path of query j was chosen, then alternative_choice[j] = i
    /// alternative_choice has size q
    pub alternative_choice: Vec<u32>,
    /// contains the probability of choosing an alternative path
    /// the probability of the i-th alternative path of query j is given by
    /// alternative_probabilities[first_alternative_of_query[j] + i]
    pub alternative_probabilities: Vec<f64>,
}

impl AlternativePathsForDTA {
    pub fn reconstruct(dir: &Path) -> Self {
        AlternativePathsForDTAFlattened::reconstruct(dir).into()
    }

    pub fn deconstruct(self, dir: &Path) -> Result<(), std::io::Error> {
        let flattened: AlternativePathsForDTAFlattened = self.into();
        flattened.deconstruct(dir)
    }
}

impl Into<AlternativePathsForDTA> for AlternativePathsForDTAFlattened {
    fn into(self) -> AlternativePathsForDTA {
        let mut alternatives = Vec::new();
        let q = self.alternative_choice.len();

        for i in 0..q {
            let start = self.first_alternative_of_query[i] as usize;
            let end = self.first_alternative_of_query[i + 1] as usize;

            let paths: Vec<AlternativePath> = (start..end)
                .map(|j| AlternativePath {
                    edges: self.egdes[self.first_edge_of_alternative[j] as usize..self.first_edge_of_alternative[j + 1] as usize]
                        .iter()
                        .map(|&e| EdgeId::from(e))
                        .collect(),
                })
                .collect();

            let costs: Vec<f64> = self.alternative_costs[start..end].to_vec();
            let probabilities: Vec<f64> = self.alternative_probabilities[start..end].to_vec();
            let choice = self.alternative_choice[i] as usize;

            alternatives.push(AlternativePaths {
                paths,
                costs,
                probabilities,
                choice,
            });
        }

        AlternativePathsForDTA { alternatives }
    }
}

impl From<AlternativePathsForDTA> for AlternativePathsForDTAFlattened {
    fn from(value: AlternativePathsForDTA) -> Self {
        let mut egdes = Vec::new();
        let mut first_alternative_of_query = Vec::new();
        let mut first_edge_of_alternative = Vec::new();
        let mut alternative_costs = Vec::new();
        let mut alternative_choice = Vec::new();
        let mut alternative_probabilities = Vec::new();

        let mut added_edges = 0;
        let mut added_alternatives = 0;

        for alternative_paths in value.alternatives.iter() {
            first_alternative_of_query.push(added_alternatives as u32);
            alternative_choice.push(alternative_paths.choice as u32);

            for (alt_index, path) in alternative_paths.paths.iter().enumerate() {
                added_alternatives += 1;
                first_edge_of_alternative.push(added_edges as u32);
                // add edges of the path
                egdes.extend(path.edges.iter().map(|&e| e));
                // add costs and probabilities
                alternative_costs.push(alternative_paths.costs[alt_index]);
                alternative_probabilities.push(alternative_paths.probabilities[alt_index]);

                // update edge index
                added_edges += path.edges.len();
            }
        }

        // add the last edge index
        first_edge_of_alternative.push(added_edges as u32);
        // add the last alternative
        first_alternative_of_query.push(added_alternatives as u32);

        Self {
            egdes,
            first_alternative_of_query,
            first_edge_of_alternative,
            alternative_costs,
            alternative_choice,
            alternative_probabilities,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into_alternate_paths_for_dta_flattened() {
        let alt_paths = AlternativePathsForDTA {
            alternatives: vec![
                AlternativePaths {
                    paths: vec![AlternativePath { edges: vec![1, 2] }, AlternativePath { edges: vec![3] }],
                    costs: vec![10.0, 20.0],
                    probabilities: vec![0.6, 0.4],
                    choice: 1,
                },
                AlternativePaths {
                    paths: vec![AlternativePath { edges: vec![4] }],
                    costs: vec![15.0],
                    probabilities: vec![1.0],
                    choice: 0,
                },
            ],
        };

        let expected = AlternativePathsForDTAFlattened {
            egdes: vec![1, 2, 3, 4],
            first_alternative_of_query: vec![0, 2, 3],
            first_edge_of_alternative: vec![0, 2, 3, 4],
            alternative_costs: vec![10.0, 20.0, 15.0],
            alternative_choice: vec![1, 0],
            alternative_probabilities: vec![0.6, 0.4, 1.0],
        };

        let flattened: AlternativePathsForDTAFlattened = alt_paths.into();
        assert_eq!(flattened.egdes, expected.egdes);
        assert_eq!(flattened.first_alternative_of_query, expected.first_alternative_of_query);
        assert_eq!(flattened.first_edge_of_alternative, expected.first_edge_of_alternative);
        assert_eq!(flattened.alternative_costs, expected.alternative_costs);
        assert_eq!(flattened.alternative_choice, expected.alternative_choice);
        assert_eq!(flattened.alternative_probabilities, expected.alternative_probabilities);
    }

    #[test]
    fn test_from_alternate_paths_for_dta_flattened() {
        let flattened = AlternativePathsForDTAFlattened {
            egdes: vec![1, 2, 3, 4],
            first_alternative_of_query: vec![0, 2, 3],
            first_edge_of_alternative: vec![0, 2, 3, 4],
            alternative_costs: vec![10.0, 20.0, 15.0],
            alternative_choice: vec![1, 0],
            alternative_probabilities: vec![0.6, 0.4, 1.0],
        };

        let expected = AlternativePathsForDTA {
            alternatives: vec![
                AlternativePaths {
                    paths: vec![AlternativePath { edges: vec![1, 2] }, AlternativePath { edges: vec![3] }],
                    costs: vec![10.0, 20.0],
                    probabilities: vec![0.6, 0.4],
                    choice: 1,
                },
                AlternativePaths {
                    paths: vec![AlternativePath { edges: vec![4] }],
                    costs: vec![15.0],
                    probabilities: vec![1.0],
                    choice: 0,
                },
            ],
        };

        let alt_paths: AlternativePathsForDTA = flattened.into();

        assert_eq!(alt_paths.alternatives.len(), expected.alternatives.len());
        for (i, alt) in alt_paths.alternatives.iter().enumerate() {
            assert_eq!(alt.paths.len(), expected.alternatives[i].paths.len());
            for (j, path) in alt.paths.iter().enumerate() {
                assert_eq!(path.edges, expected.alternatives[i].paths[j].edges);
            }
            assert_eq!(alt.costs, expected.alternatives[i].costs);
            assert_eq!(alt.probabilities, expected.alternatives[i].probabilities);
            assert_eq!(alt.choice, expected.alternatives[i].choice);
        }
    }
}
