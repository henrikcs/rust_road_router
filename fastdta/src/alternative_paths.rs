use std::path::Path;

use conversion::{
    FILE_DTA_QUERIES_ALTERNATIVE_CHOICE, FILE_DTA_QUERIES_ALTERNATIVE_COST, FILE_DTA_QUERIES_ALTNERNATIVE_PROBABILITIES, FILE_DTA_QUERIES_EDGE_IDS,
    FILE_DTA_QUERIES_FIRST_ALTERNATIVE_PATH, FILE_DTA_QUERIES_FIRST_EDGE_OF_ALTERNATIVE, SerializedTimestamp,
};
use rand::{
    SeedableRng,
    distr::{Distribution, weighted::WeightedIndex},
    rngs::StdRng,
};
use rust_road_router::{
    datastr::graph::{
        EdgeId,
        floating_time_dependent::{FlWeight, TDGraph, Timestamp},
    },
    io::{Load, Store},
};

use crate::choice::ChoiceAlgorithm;
use crate::gawron::gawron;
use crate::logit::logit;

#[derive(Debug, Clone)]
pub struct AlternativePathsForDTA {
    /// queries i has alternatives alternatives[i]
    pub alternatives_in_query: Vec<AlternativePaths>,
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
    pub fn perform_choice_model(&mut self, choice_algorithm: &ChoiceAlgorithm, max_alternatives: u32, previous_costs: &Vec<f64>) {
        // Calculate probabilities using the choice algorithm
        self.apply_choice_algorithm(&choice_algorithm, previous_costs);

        // Check if we have more alternatives than allowed and adapt alternatives with probabilities if necessary
        if self.paths.len() > max_alternatives as usize {
            self.remove_least_probable_routes(max_alternatives, choice_algorithm);
        }
    }

    /// Remove routes with lowest probability and rescale
    fn remove_least_probable_routes(&mut self, max_alternatives: u32, _choice_algorithm: &ChoiceAlgorithm) {
        // Create vector of (probability, index) pairs
        let mut prob_with_indices: Vec<(f64, usize)> = self.probabilities.iter().enumerate().map(|(index, &p)| (p, index)).collect();

        // Sort by probability (ascending)
        prob_with_indices.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        // Get indices to remove (lowest probability routes)
        let indices_to_remove: Vec<usize> = prob_with_indices
            .iter()
            .take(prob_with_indices.len() - max_alternatives as usize)
            .map(|&(_, index)| index)
            .collect();

        // Remove routes with lowest probabilities
        let mut new_paths = Vec::new();
        let mut new_costs = Vec::new();
        let mut new_probabilities = Vec::new();

        for (i, path) in self.paths.iter().enumerate() {
            if !indices_to_remove.contains(&i) {
                new_paths.push(path.clone());
                new_costs.push(self.costs[i]);
                new_probabilities.push(self.probabilities[i]);
            }
        }

        self.paths = new_paths;
        self.costs = new_costs;
        self.probabilities = new_probabilities;

        // Rescale probabilities to sum to 1
        let prob_sum: f64 = self.probabilities.iter().sum();
        if prob_sum > 0.0 {
            for prob in self.probabilities.iter_mut() {
                *prob /= prob_sum;
            }
        }
    }

    pub fn choose(&mut self, rng: &mut StdRng) {
        if self.paths.is_empty() {
            return;
        }

        let prob = match WeightedIndex::new(&self.probabilities) {
            Ok(p) => p,
            Err(err) => {
                println!("Error creating WeightedIndex: {}", err);
                println!("Probabilities: {:?}", self.probabilities);
                println!("Costs: {:?}", self.costs);
                println!("Paths: {:?}", self.paths);
                println!("Choice: {:?}", self.choice);

                panic!("Failed to create WeightedIndex for choosing alternative path.");
            }
        };
        self.choice = prob.sample(rng);
    }

    pub fn apply_choice_algorithm(&mut self, choice_algorithm: &ChoiceAlgorithm, previous_costs: &Vec<f64>) {
        match choice_algorithm {
            ChoiceAlgorithm::Gawron { a, beta } => {
                self.probabilities = gawron(self, *a, *beta, previous_costs);
            }
            ChoiceAlgorithm::Logit { beta, gamma, theta } => {
                self.probabilities = logit(self, *beta, *gamma, *theta);
            }
        }
    }

    pub fn scale_probabilities(&mut self, scale: f64) {
        for prob in &mut self.probabilities {
            *prob *= scale;
        }
    }
}

/// path represented by a sequence of edge id used in TDGraph
#[derive(Debug, Clone)]
pub struct AlternativePath {
    pub edges: Vec<EdgeId>,
}

impl AlternativePathsForDTA {
    pub fn perform_choice_model(
        &mut self,
        previous_alternatives: &Self,
        choice_algorithm: &ChoiceAlgorithm,
        max_alternatives: u32,
        keep_routes: &Vec<bool>,
        seed: i32,
    ) {
        let mut rng: StdRng = StdRng::seed_from_u64(seed.abs() as u64);

        for (i, alternative_paths) in self.alternatives_in_query.iter_mut().enumerate() {
            let previous_costs = &previous_alternatives.alternatives_in_query[i].costs;

            alternative_paths.perform_choice_model(choice_algorithm, max_alternatives, previous_costs);

            // decide if the route may change or not by checking "keep_route_probabilities[i]"
            if keep_routes[i] {
                continue;
            }

            alternative_paths.choose(&mut rng);
        }
    }

    /// Initialize an empty AlternativePathsForDTA with no alternatives for each query.
    /// This is useful when starting from scratch (iteration 0) and paths will be added
    /// via update_alternatives_with_new_paths.
    pub fn init_empty(num_queries: usize) -> Self {
        AlternativePathsForDTA {
            alternatives_in_query: (0..num_queries)
                .map(|_| AlternativePaths {
                    paths: vec![],
                    costs: vec![],
                    probabilities: vec![],
                    choice: 0,
                })
                .collect(),
        }
    }

    pub fn init(shortest_paths: &Vec<Vec<u32>>, travel_times: &Vec<FlWeight>) -> Self {
        debug_assert_eq!(
            shortest_paths.len(),
            travel_times.len(),
            "shortest_paths and travel_times must have the same length",
        );

        AlternativePathsForDTA {
            alternatives_in_query: shortest_paths
                .iter()
                .enumerate()
                .map(|(i, path)| AlternativePaths {
                    paths: vec![AlternativePath {
                        edges: path.iter().map(|&e| e as EdgeId).collect(),
                    }],
                    costs: vec![travel_times[i].into()],
                    probabilities: vec![1.0],
                    choice: 0,
                })
                .collect(),
        }
    }

    /// merges the previous alternatives with the current shortest paths
    pub fn update_alternatives_with_new_paths(
        &self,
        shortest_paths: &Vec<Vec<EdgeId>>,
        travel_times: &Vec<FlWeight>,
        departures: &Vec<SerializedTimestamp>,
        graph: &TDGraph,
    ) -> AlternativePathsForDTA {
        let mut merged_alternative_paths = self.clone();

        // merge previous alternatives with current paths
        for (i, alternatives) in merged_alternative_paths.alternatives_in_query.iter_mut().enumerate() {
            let mut is_shortest_path_among_alternatives = false;

            // Update costs for existing paths
            for (j, alternative_path) in alternatives.paths.iter().enumerate() {
                if alternative_path.edges == shortest_paths[i].iter().map(|&e| e as EdgeId).collect::<Vec<EdgeId>>() {
                    // path already exists, mark it and update its cost
                    is_shortest_path_among_alternatives = true;
                    alternatives.costs[j] = travel_times[i].into();
                } else {
                    // Recompute cost for existing alternative path
                    alternatives.costs[j] = graph
                        .get_travel_time_along_path(Timestamp::from_millis(departures[i]), &alternative_path.edges)
                        .into();

                    if alternatives.costs[j] < 0.0 {
                        println!(
                            "Warning: Negative travel time encountered for alternative path in query {}. Setting cost to infinity.",
                            i
                        );

                        alternatives.costs[j] = f64::INFINITY;
                    }
                }
            }

            // Add new shortest path if it's not among existing alternatives
            if !is_shortest_path_among_alternatives {
                alternatives.paths.push(AlternativePath {
                    edges: shortest_paths[i].iter().map(|&e| e as EdgeId).collect(),
                });
                alternatives.costs.push(travel_times[i].into());

                let scale = (alternatives.paths.len() - 1) as f64 / alternatives.paths.len() as f64;
                alternatives.scale_probabilities(scale);

                // Extend probabilities vector with initial probability for new route
                alternatives.probabilities.push(1.0 / alternatives.paths.len() as f64);
            }
        }

        merged_alternative_paths
    }

    pub fn get_chosen_paths<'a>(&'a self) -> Vec<&'a Vec<EdgeId>> {
        let mut chosen_paths = vec![];

        for alternatives in self.alternatives_in_query.iter() {
            if alternatives.paths.is_empty() {
                // Return reference to a static empty vec for empty alternatives
                static EMPTY_VEC: Vec<EdgeId> = Vec::new();
                chosen_paths.push(&EMPTY_VEC);
            } else {
                let choice_index = alternatives.choice;
                let chosen_path = &alternatives.paths[choice_index];
                chosen_paths.push(&chosen_path.edges);
            }
        }

        chosen_paths
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
        if !dir.exists() {
            std::fs::create_dir(dir)?;
        }
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

        AlternativePathsForDTA {
            alternatives_in_query: alternatives,
        }
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

        for alternative_paths in value.alternatives_in_query.iter() {
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
            alternatives_in_query: vec![
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
            alternatives_in_query: vec![
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

        assert_eq!(alt_paths.alternatives_in_query.len(), expected.alternatives_in_query.len());
        for (i, alt) in alt_paths.alternatives_in_query.iter().enumerate() {
            assert_eq!(alt.paths.len(), expected.alternatives_in_query[i].paths.len());
            for (j, path) in alt.paths.iter().enumerate() {
                assert_eq!(path.edges, expected.alternatives_in_query[i].paths[j].edges);
            }
            assert_eq!(alt.costs, expected.alternatives_in_query[i].costs);
            assert_eq!(alt.probabilities, expected.alternatives_in_query[i].probabilities);
            assert_eq!(alt.choice, expected.alternatives_in_query[i].choice);
        }
    }

    #[test]
    fn test_perform_choice_model_without_new_route() {
        let choice_algorithm = ChoiceAlgorithm::Gawron { a: 0.1, beta: 0.7 };
        let mut alternatives = AlternativePaths {
            paths: vec![AlternativePath { edges: vec![1, 2] }, AlternativePath { edges: vec![3, 4] }],
            costs: vec![10.0, 15.0],
            probabilities: vec![0.6, 0.4],
            choice: 0,
        };
        let old_cost = vec![8.0, 12.0];

        alternatives.perform_choice_model(&choice_algorithm, 5, &old_cost);

        assert_eq!(alternatives.costs[0], 10.0);

        // Cost smoothing DOES apply to non-chosen route (route 1)
        let expected_cost_1 = 0.7 * 15.0 + 0.3 * 12.0; // beta * new_cost + (1-beta) * old_cost = 10.5 + 3.6 = 14.1
        assert!((alternatives.costs[1] - expected_cost_1).abs() < 1e-10);

        // Probabilities should have been recalculated
        assert_eq!(alternatives.probabilities.len(), 2);
        let prob_sum: f64 = alternatives.probabilities.iter().sum();
        assert!((prob_sum - 1.0).abs() < 0.001); // Should sum to 1
    }

    #[test]
    fn test_remove_least_probable_routes() {
        let choice_algorithm = ChoiceAlgorithm::Gawron { a: 0.1, beta: 0.7 };
        let mut alternatives = AlternativePaths {
            paths: vec![
                AlternativePath { edges: vec![1, 2] },
                AlternativePath { edges: vec![3, 4] },
                AlternativePath { edges: vec![5, 6] },
                AlternativePath { edges: vec![7, 8] },
            ],
            costs: vec![10.0, 15.0, 20.0, 25.0],
            probabilities: vec![0.4, 0.3, 0.2, 0.1], // Route 3 has lowest probability
            choice: 0,
        };

        let previous_costs = vec![10.0, 15.0, 20.0, 25.0];

        alternatives.perform_choice_model(&choice_algorithm, 3, &previous_costs);

        // Should have only 3 routes after removal
        assert_eq!(alternatives.paths.len(), 3);
        assert_eq!(alternatives.costs.len(), 3);
        assert_eq!(alternatives.probabilities.len(), 3);

        // Route with lowest probability should be removed
        // (Note: exact routes depend on Gawron calculation, but we check basic constraints)
        let prob_sum: f64 = alternatives.probabilities.iter().sum();
        assert!((prob_sum - 1.0).abs() < 0.001); // Should sum to 1
    }

    #[test]
    fn test_single_route_probabilities() {
        let choice_algorithm = ChoiceAlgorithm::Gawron { a: 0.1, beta: 0.7 };
        let mut alternatives = AlternativePaths {
            paths: vec![AlternativePath { edges: vec![1, 2] }],
            costs: vec![10.0],
            probabilities: vec![1.0],
            choice: 0,
        };

        let previous_costs = vec![10.0];

        alternatives.perform_choice_model(&choice_algorithm, 5, &previous_costs);

        // Single route should have probability 1.0
        assert_eq!(alternatives.probabilities.len(), 1);
        assert_eq!(alternatives.probabilities[0], 1.0);
    }

    #[test]
    fn test_alternative_paths_for_dta_perform_choice_model() {
        let choice_algorithm = ChoiceAlgorithm::Gawron { a: 0.1, beta: 0.7 };

        let previous_alternatives = AlternativePathsForDTA {
            alternatives_in_query: vec![AlternativePaths {
                paths: vec![AlternativePath { edges: vec![1, 2] }],
                costs: vec![8.0],
                probabilities: vec![1.0],
                choice: 0,
            }],
        };

        let mut current_alternatives = AlternativePathsForDTA {
            alternatives_in_query: vec![AlternativePaths {
                paths: vec![AlternativePath { edges: vec![1, 2] }, AlternativePath { edges: vec![3, 4] }],
                costs: vec![10.0, 15.0],
                probabilities: vec![0.6, 0.4],
                choice: 0,
            }],
        };

        current_alternatives.perform_choice_model(&previous_alternatives, &choice_algorithm, 5, &vec![false], 123);

        // Check that choice was made (0 or 1)
        assert!(current_alternatives.alternatives_in_query[0].choice < 2);

        // Check that probabilities sum to 1
        let prob_sum: f64 = current_alternatives.alternatives_in_query[0].probabilities.iter().sum();
        assert!((prob_sum - 1.0).abs() < 0.001);
    }
}
