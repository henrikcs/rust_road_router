pub trait TrafficModel {
    fn get_speed(&self, density: f64) -> f64;
    fn calibrate(&mut self, observed_speed: &[f64], observed_density: &[f64]);
}

pub mod modified_lee {

    use crate::traffic_model::TrafficModel;

    const MIN_IMPROVEMENT_PERCENT: f64 = 0.01;
    const MAX_CALIBRATION_ITERATIONS: usize = 10;
    const MAX_JAM_DENSITY: f64 = 167.0; // vehicles per km per lane (= 6m per vehicle)

    #[derive(Debug, Clone)]
    pub struct ModifiedLeeRanges {
        pub a: Vec<f64>,
        pub e: Vec<f64>,
        pub theta: Vec<f64>,
        pub jam_density: Vec<f64>,
    }

    impl ModifiedLeeRanges {
        pub fn init(min_jam_density: f64) -> Self {
            Self::new(0.001, 5.0, 0.0, 5.0, 0.001, 5.0, min_jam_density, MAX_JAM_DENSITY)
        }

        pub fn new(a_min: f64, a_max: f64, e_min: f64, e_max: f64, theta_min: f64, theta_max: f64, jam_density_min: f64, jam_density_max: f64) -> Self {
            Self {
                a: vec![a_min, a_max],
                e: vec![e_min, e_max],
                theta: vec![theta_min, theta_max],
                jam_density: vec![jam_density_min, jam_density_max],
            }
        }

        pub fn divide(&self) -> Vec<ModifiedLeeRanges> {
            // TODO: divide only if the range is larger than a minimum threshold
            // let minimum_threshold = 1e-3;

            let a_mid = (self.a[0] + self.a[1]) / 2.0;
            let e_mid = (self.e[0] + self.e[1]) / 2.0;
            let theta_mid = (self.theta[0] + self.theta[1]) / 2.0;
            let jam_density_mid = (self.jam_density[0] + self.jam_density[1]) / 2.0;

            let mut divided_ranges = Vec::new();

            for a_range in &[(self.a[0], a_mid), (a_mid, self.a[1])] {
                for e_range in &[(self.e[0], e_mid), (e_mid, self.e[1])] {
                    for theta_range in &[(self.theta[0], theta_mid), (theta_mid, self.theta[1])] {
                        for jam_density_range in &[(self.jam_density[0], jam_density_mid), (jam_density_mid, self.jam_density[1])] {
                            divided_ranges.push(ModifiedLeeRanges {
                                a: vec![a_range.0, a_range.1],
                                e: vec![e_range.0, e_range.1],
                                theta: vec![theta_range.0, theta_range.1],
                                jam_density: vec![jam_density_range.0, jam_density_range.1],
                            });
                        }
                    }
                }
            }

            divided_ranges
        }

        pub fn get_a_mid(&self) -> f64 {
            (self.a[0] + self.a[1]) / 2.0
        }

        pub fn get_e_mid(&self) -> f64 {
            (self.e[0] + self.e[1]) / 2.0
        }

        pub fn get_theta_mid(&self) -> f64 {
            (self.theta[0] + self.theta[1]) / 2.0
        }

        pub fn get_jam_density_mid(&self) -> f64 {
            (self.jam_density[0] + self.jam_density[1]) / 2.0
        }

        pub fn get_a_diff(&self) -> f64 {
            self.a[1] - self.a[0]
        }

        pub fn get_e_diff(&self) -> f64 {
            self.e[1] - self.e[0]
        }

        pub fn get_theta_diff(&self) -> f64 {
            self.theta[1] - self.theta[0]
        }

        pub fn get_jam_density_diff(&self) -> f64 {
            self.jam_density[1] - self.jam_density[0]
        }
    }

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
            let ranges = ModifiedLeeRanges::init(jam_density_min);

            Self {
                free_flow_speed,
                jam_density_min,
                jam_density: ranges.get_jam_density_mid(),
                theta: ranges.get_theta_mid(),
                a: ranges.get_a_mid(),
                e: ranges.get_e_mid(),
            }
        }

        fn f(density: f64, free_flow_speed: f64, a: f64, e: f64, theta: f64, jam_density: f64) -> f64 {
            let d = density / jam_density;
            free_flow_speed * (1.0 - (d).powf(a)) / (1.0 + e * d.powf(theta))
        }

        fn f_prime(density: f64, free_flow_speed: f64, a: f64, e: f64, theta: f64, jam_density: f64) -> f64 {
            let d = density / jam_density;
            let df_ddensity = -free_flow_speed * a * (density).powf(a - 1.0) / (1.0 + e * d.powf(theta))
                + free_flow_speed * (1.0 - (density).powf(a)) * e * theta * d.powf(theta - 1.0) / (jam_density * (1.0 + e * d.powf(theta)).powi(2));
            df_ddensity
        }

        /// if positive, the stable shock wave property is not violated
        /// if non-positive, the stable shock wave property is violated
        fn stable_shock_wave_property(a: f64, e: f64, theta: f64) -> f64 {
            (2.0 * e * theta) / (e + 1.0) - a + 1.0
        }

        fn calibrate_with_ranges(&mut self, observed_speed: &[f64], observed_density: &[f64], initial_ranges: &mut ModifiedLeeRanges) {
            let cost_function = ModifiedLeeCostFunction::new(self.free_flow_speed, observed_speed, observed_density);

            let mut best_sse = cost_function.sse(self.a, self.e, self.theta, self.jam_density);
            let mut converged = false;

            let mut iteration = 0;

            while iteration < MAX_CALIBRATION_ITERATIONS && !converged {
                let mut best_sse_in_loop = best_sse;
                // divide the ranges into two smaller sub-ranges each
                // this results in 2^4 = 16 combinations of parameter ranges
                let divided_ranges = initial_ranges.divide();

                let mut improved = false;
                // for each combination of parameter ranges, check the sse
                for r in divided_ranges {
                    let a_bounds = r.a;
                    let e_bounds = r.e;
                    let theta_bounds = r.theta;
                    let jam_density_bounds = r.jam_density;

                    let a = (a_bounds[0] + a_bounds[1]) / 2.0;
                    let e = (e_bounds[0] + e_bounds[1]) / 2.0;
                    let theta = (theta_bounds[0] + theta_bounds[1]) / 2.0;
                    let jam_density = (jam_density_bounds[0] + jam_density_bounds[1]) / 2.0;

                    // check stable shock wave property
                    if Self::stable_shock_wave_property(a, e, theta) <= 0.0 {
                        continue;
                    }

                    let sse = cost_function.sse(a, e, theta, jam_density);
                    if sse < best_sse_in_loop {
                        best_sse_in_loop = sse;

                        self.a = a;
                        self.e = e;
                        self.theta = theta;
                        self.jam_density = jam_density;

                        initial_ranges.a = a_bounds;
                        initial_ranges.e = e_bounds;
                        initial_ranges.theta = theta_bounds;
                        initial_ranges.jam_density = jam_density_bounds;

                        improved = true;
                    }
                }

                if improved == false {
                    // no improvement found in this iteration
                    break;
                }

                // if there is only an improvement by 1%, stop
                if best_sse - best_sse_in_loop < MIN_IMPROVEMENT_PERCENT * best_sse {
                    converged = true;
                }

                best_sse = best_sse_in_loop;

                iteration += 1;
            }
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

            let mut ranges: ModifiedLeeRanges = ModifiedLeeRanges::init(self.jam_density_min);
            let mut original_ranges = ranges.clone();

            let mut converged = false;

            while converged == false {
                self.calibrate_with_ranges(observed_speed, observed_density, &mut ranges);

                // println!(
                //     "Current Parameters after calibration: a = {}, e = {}, theta = {}, jam_density = {}",
                //     self.a, self.e, self.theta, self.jam_density
                // );

                converged = true;

                // if ranges are close to the upper bound (within 1% of the range), re-intialize ranges with shifted initial upper bound

                if (original_ranges.a[1] - self.a) / original_ranges.get_a_diff() < 0.01 {
                    ranges.a[0] = original_ranges.a[1];
                    ranges.a[1] = original_ranges.get_a_diff() + original_ranges.a[1];
                    converged = false;
                }

                if (original_ranges.e[1] - self.e) / original_ranges.get_e_diff() < 0.01 {
                    ranges.e[0] = original_ranges.e[1];
                    ranges.e[1] = original_ranges.get_e_diff() + original_ranges.e[1];
                    converged = false;
                }

                if (original_ranges.theta[1] - self.theta) / original_ranges.get_theta_diff() < 0.01 {
                    ranges.theta[0] = original_ranges.theta[1];
                    ranges.theta[1] = original_ranges.get_theta_diff() + original_ranges.theta[1];
                    converged = false;
                }

                if (original_ranges.jam_density[1] - self.jam_density) / original_ranges.get_jam_density_diff() < 0.01 {
                    ranges.jam_density[0] = original_ranges.jam_density[1];
                    ranges.jam_density[1] = original_ranges.get_jam_density_diff() + original_ranges.jam_density[1];
                    converged = false;
                }

                if converged == false {
                    original_ranges = ranges.clone();

                    // println!("Re-initializing calibration ranges to {:?}", ranges);
                }
            }

            // panic!(
            //     "Calibrated Modified Lee parameters: a = {}, e = {}, theta = {}, jam_density = {}",
            //     self.a, self.e, self.theta, self.jam_density
            // );
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

        pub fn sse(&self, a: f64, e: f64, theta: f64, jam_density: f64) -> f64 {
            self.observed_density
                .iter()
                .enumerate()
                .map(|(i, density)| {
                    let speed = ModifiedLee::f(*density, self.free_flow_speed, a, e, theta, jam_density);
                    let observed = self.observed_speed[i];
                    let error = speed - observed;
                    error * error
                })
                .sum()
        }
    }

    mod tests {

        use super::*;

        #[test]
        fn test_modified_lee_speed_positive() {
            let model = ModifiedLee::new(13.6, 60.0);

            let speed = model.get_speed(2.0);
            dbg!(&speed);
            assert!(speed > 0.0);
        }

        #[test]
        fn test_free_flow_speed() {
            let model = ModifiedLee::new(13.6, 30.0);

            let speed = model.get_speed(0.0);
            dbg!(&speed);
            assert!((speed - 13.6).abs() < 1e-6);
        }

        #[test]
        fn test_calibrated_values() {
            let model = ModifiedLee {
                a: 2.57,
                e: 3.10,
                theta: 2.23,
                jam_density: 60.0,
                jam_density_min: 30.0,
                free_flow_speed: 109.0,
            };

            let speed = model.get_speed(45.0);
            dbg!(&speed);
            assert!(speed > 0.0);
        }
    }
}
