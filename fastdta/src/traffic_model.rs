pub trait TrafficModel {
    fn get_speed(&self, density: f64) -> f64;
    fn calibrate(&mut self, observed_speed: &[f64], observed_density: &[f64]);
}

pub mod modified_lee {
    use crate::traffic_model::TrafficModel;
    use argmin::{
        core::{CostFunction, Executor, State},
        solver::particleswarm::ParticleSwarm,
    };

    const MAX_CALIBRATION_ITERATIONS: u64 = 10;
    const MAX_JAM_DENSITY: f64 = 167.0; // vehicles per km per lane (= 6m per vehicle)

    const MIN_A: f64 = 0.001;
    const MAX_A: f64 = 5.0;

    const MIN_E: f64 = 0.001;
    const MAX_E: f64 = 5.0;

    const MIN_THETA: f64 = 0.0;
    const MAX_THETA: f64 = 10.0;

    pub struct ModifiedLee {
        free_flow_speed: f64,
        jam_density_min: f64,
        jam_density: f64,
        theta: f64,
        a: f64,
        e: f64,
    }

    impl ModifiedLee {
        pub fn new(free_flow_speed: f64, jam_density_min: f64) -> Self {
            Self {
                free_flow_speed,
                jam_density_min,
                jam_density: (jam_density_min + MAX_JAM_DENSITY) / 2.0,
                theta: (MIN_THETA + MAX_THETA) / 2.0,
                a: (MIN_A + MAX_A) / 2.0,
                e: (MIN_E + MAX_E) / 2.0,
            }
        }

        fn f(density: f64, free_flow_speed: f64, a: f64, e: f64, theta: f64, jam_density: f64) -> f64 {
            let d = density / jam_density;
            free_flow_speed * (1.0 - (density).powf(a)) / (1.0 + e * d.powf(theta))
        }
    }

    impl TrafficModel for ModifiedLee {
        fn calibrate(&mut self, observed_speed: &[f64], observed_density: &[f64]) {
            // observerd_speed and observed_density are vectors of the same length
            debug_assert!(observed_speed.len() == observed_density.len());
            // iterate calibration until SSE reaches minimum

            // adapt variables a, e, theta, jam_density to minimize SSE such that
            // q_max = f(k_m) * k_m \in [0.75 * inititial_capacity, 1.25 * intial_capacity],
            // where k_m is the optimal density, i.e. f_prime(k_m) = 0

            let lower_bounds = vec![0.001, 0.001, 0.0, self.jam_density_min];
            let upper_bounds = vec![5.0, 5.0, 10.0, MAX_JAM_DENSITY];

            let population_size = 50;
            let solver = ParticleSwarm::new((lower_bounds, upper_bounds), population_size);
            let cf: ModifiedLeeCostFunction = ModifiedLeeCostFunction::new(self.free_flow_speed, &observed_speed, &observed_density);
            let res = Executor::new(cf, solver)
                .configure(|state| state.max_iters(MAX_CALIBRATION_ITERATIONS))
                .run()
                .unwrap();

            let best_param = res.state().get_best_param().unwrap();

            self.a = best_param.position[0];
            self.e = best_param.position[1];
            self.theta = best_param.position[2];
            self.jam_density = best_param.position[3];
        }

        fn get_speed(&self, density: f64) -> f64 {
            Self::f(density, self.free_flow_speed, self.a, self.e, self.theta, self.jam_density)
        }
    }

    struct ModifiedLeeCostFunction<'a> {
        free_flow_speed: f64,
        observed_speed: &'a [f64],
        observed_density: &'a [f64],
    }

    impl<'a> ModifiedLeeCostFunction<'a> {
        pub fn new(free_flow_speed: f64, observed_speed: &'a [f64], observed_density: &'a [f64]) -> Self {
            Self {
                free_flow_speed,
                observed_speed,
                observed_density,
            }
        }
    }

    impl<'a> CostFunction for ModifiedLeeCostFunction<'a> {
        type Param = Vec<f64>;
        type Output = f64;

        // minimize sum of squared errors between observed speeds and speeds predicted by Modified Lee model
        fn cost(&self, param: &Self::Param) -> Result<Self::Output, argmin::core::Error> {
            // param[0] = a
            // param[1] = e
            // param[2] = theta
            // param[3] = jam_density

            let a = param[0];
            let e = param[1];
            let theta = param[2];
            let jam_density = param[3];

            let sse: f64 = self
                .observed_density
                .iter()
                .enumerate()
                .map(|(i, density)| {
                    let speed = ModifiedLee::f(*density, self.free_flow_speed, a, e, theta, jam_density);
                    let observed = self.observed_speed[i];
                    let error = speed - observed;
                    error * error
                })
                .sum();

            Ok(sse)
        }
    }
}
