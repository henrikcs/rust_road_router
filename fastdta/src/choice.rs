use rand::{
    SeedableRng,
    distr::{Distribution, weighted::WeightedIndex},
    rngs::StdRng,
};

pub struct Choices {
    pub choice_sets: Vec<Vec<u32>>,
    pub weights: Vec<Vec<f64>>,
    pub probabilities: Vec<Vec<f64>>,
    pub choices: Vec<usize>,
}

impl Choices {
    pub fn create(choice_sets: Vec<Vec<u32>>, weights: Vec<Vec<f64>>, choice_algorithm: AlternativePathsChoiceAlgorithm, seed: u64) -> Self {
        debug_assert_eq!(choice_sets.len(), weights.len(), "Choice sets and weight lists must have the same length");
        let mut probabilities = Vec::with_capacity(choice_sets.len());
        let mut choices = Vec::with_capacity(choice_sets.len());

        let mut rng: StdRng = StdRng::seed_from_u64(seed);

        for (i, choice_set) in choice_sets.iter().enumerate() {
            debug_assert_eq!(choice_set.len(), weights[i].len(), "Choice set and weights must have the same length");
            debug_assert!(!choice_set.is_empty(), "Choice set must not be empty");
            let (prob, choice) = match choice_algorithm {
                AlternativePathsChoiceAlgorithm::Gawron { a, beta } => gawron(&mut rng, choice_set, &weights[i], a, beta),
                AlternativePathsChoiceAlgorithm::Logit { theta } => logit(&mut rng, &weights[i], theta),
            };

            probabilities.push(prob);
            choices.push(choice);
        }

        Self {
            choice_sets,
            weights,
            probabilities,
            choices,
        }
    }
}

pub enum AlternativePathsChoiceAlgorithm {
    Gawron { a: f64, beta: f64 },
    Logit { theta: f64 },
}

fn gawron(rng: &mut StdRng, choice_set: &Vec<u32>, weights: &Vec<f64>, a: f64, beta: f64) -> (Vec<f64>, usize) {
    let mut probabilities = vec![0.0; choice_set.len()];

    // TODO: Implement the Gawron choice model

    // for this, we need the previous iteration's travel times, previous probability of chosen route
    // and the current iteration's travel times

    (probabilities, 0)
}

fn logit(rng: &mut StdRng, costs: &Vec<f64>, theta: f64) -> (Vec<f64>, usize) {
    let mut probabilities = vec![0.0; costs.len()];

    let mut sum_weights: f64 = 0.0;

    for &weight in costs {
        sum_weights += (theta * weight).exp();
    }

    for (i, &cost) in costs.iter().enumerate() {
        probabilities[i] = (theta * cost).exp() / sum_weights;
    }

    // choose randomly from the probabilities
    // the vec probabilities defines a probability distribution
    let distr = WeightedIndex::new(&probabilities);

    (probabilities, distr.unwrap().sample(rng))
}
