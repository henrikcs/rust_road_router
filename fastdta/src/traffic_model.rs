pub trait TrafficModel: Send + Sync {
    /// speed in km/h for given density in vehicles per km per lane
    fn get_speed(&self, density: f64) -> f64;

    fn calibrate(&mut self, observed_speed: &[f64], observed_density: &[f64]);

    // print debug information about the traffic model
    fn debug(&self);

    /// convert the traffic model parameters to a vector
    fn get_params_as_vec(&self) -> Vec<f64>;

    fn from_vec(params: &Vec<f64>) -> Self
    where
        Self: Sized;
}

#[derive(Debug, Clone)]
pub enum TrafficModelType {
    ModifiedLee,
}

pub mod modified_lee {

    use nlopt::{Algorithm, Nlopt, Target};

    use crate::traffic_model::TrafficModel;

    const MIN_A: f64 = 0.000_1;
    const MIN_E: f64 = 0.0;
    const MIN_THETA: f64 = 0.000_1;

    const MAX_A: f64 = 5.0;
    const MAX_E: f64 = 10.0;
    const MAX_THETA: f64 = 5.0;
    const MAX_JAM_DENSITY: f64 = 166.66; // vehicles per km per lane (= 6m per vehicle)
    const MIN_JAM_DENSITY: f64 = 50.0; // vehicles per km per lane (= 20m per vehicle)

    #[derive(Debug, Clone)]
    pub struct ModifiedLeeRanges {
        pub a: Vec<f64>,
        pub e: Vec<f64>,
        pub theta: Vec<f64>,
        pub jam_density: Vec<f64>,
    }

    impl ModifiedLeeRanges {
        pub fn init(min_jam_density: f64) -> Self {
            Self::new(MIN_A, MAX_A, MIN_E, MAX_E, MIN_THETA, MAX_THETA, min_jam_density, MAX_JAM_DENSITY)
        }

        pub fn new(a_min: f64, a_max: f64, e_min: f64, e_max: f64, theta_min: f64, theta_max: f64, jam_density_min: f64, jam_density_max: f64) -> Self {
            Self {
                a: vec![a_min, a_max],
                e: vec![e_min, e_max],
                theta: vec![theta_min, theta_max],
                jam_density: vec![jam_density_min, jam_density_max],
            }
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
            let ranges = ModifiedLeeRanges::init(f64::max(jam_density_min, MIN_JAM_DENSITY));

            Self {
                free_flow_speed,
                jam_density_min: f64::max(jam_density_min, MIN_JAM_DENSITY),
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

        pub fn calibrate(&mut self, observed_speed: &[f64], observed_density: &[f64]) {
            // use nlopt to calibrate the parameters

            let objective_function = |x: &[f64], mut grad: Option<&mut [f64]>, _params: &mut [f64; 0]| {
                let (a, e, theta, k) = (x[0], x[1], x[2], x[3]);
                let mut sse = 0.0;

                // initialize gradient to zero
                if let Some(g) = grad.as_deref_mut() {
                    g.iter_mut().for_each(|v| *v = 0.0);
                }

                for (x, u) in observed_density.iter().zip(observed_speed.iter()) {
                    let d = x / k;
                    let da = d.powf(a);
                    let dtheta = d.powf(theta);
                    let denom = 1.0 + e * dtheta;
                    let num = 1.0 - da;
                    let f = self.free_flow_speed * num / denom;

                    let r = f - u;
                    sse += r * r;

                    if let Some(g) = grad.as_deref_mut() {
                        // partials of f (as above)
                        let df_da = -self.free_flow_speed * da * d.ln() / denom;
                        let df_de = -self.free_flow_speed * num * dtheta / (denom * denom);
                        let df_dt = -self.free_flow_speed * num * e * dtheta * d.ln() / (denom * denom);
                        let df_dk = self.free_flow_speed / k * (a * da * (1.0 + e * dtheta) + e * theta * dtheta * (1.0 - da)) / (denom * denom);

                        g[0] += 2.0 * r * df_da;
                        g[1] += 2.0 * r * df_de;
                        g[2] += 2.0 * r * df_dt;
                        g[3] += 2.0 * r * df_dk;
                    }
                }

                sse
            };

            let mut opt = Nlopt::new(Algorithm::Mma, 4, objective_function, Target::Minimize, []);

            let lb = [MIN_A, MIN_E, MIN_THETA, self.jam_density_min]; // lower bounds
            let ub = [MAX_A, MAX_E, MAX_THETA, MAX_JAM_DENSITY];
            opt.set_lower_bounds(&lb).unwrap();
            opt.set_upper_bounds(&ub).unwrap();

            let stable_shockwave_property_constraint = |x: &[f64], grad: Option<&mut [f64]>, _data: &mut ()| {
                let (a, e, t, _) = (x[0], x[1], x[2], x[3]);
                if let Some(g) = grad {
                    g[0] = -1.0;
                    g[1] = 2.0 * t / (e + 1.0).powi(2);
                    g[2] = 2.0 * e / (e + 1.0);
                    g[3] = 0.0;
                }
                2.0 * e * t / (e + 1.0) - a + 1.0
            };

            opt.add_inequality_constraint(stable_shockwave_property_constraint, (), 1.0e-8).unwrap();

            opt.set_ftol_abs(1.0e-5).unwrap();
            opt.set_maxtime(0.1).unwrap();

            let mut x = [self.a, self.e, self.theta, self.jam_density];
            let res = opt.optimize(&mut x);
            if let Err(e) = &res {
                match e.0 {
                    nlopt::FailState::RoundoffLimited | nlopt::FailState::ForcedStop => {}
                    _ => {
                        println!("NLopt optimization failed: {:?}", e);
                    }
                }
            }

            self.a = x[0];
            self.e = x[1];
            self.theta = x[2];
            self.jam_density = x[3];
        }
    }

    impl TrafficModel for ModifiedLee {
        fn calibrate(&mut self, observed_speed: &[f64], observed_density: &[f64]) {
            self.calibrate(observed_speed, observed_density);

            // panic!(
            //     "Calibrated Modified Lee parameters: a = {}, e = {}, theta = {}, jam_density = {}",
            //     self.a, self.e, self.theta, self.jam_density
            // );
        }

        fn get_speed(&self, density: f64) -> f64 {
            Self::f(density, self.free_flow_speed, self.a, self.e, self.theta, self.jam_density)
        }

        fn debug(&self) {
            println!(
                "Modified Lee Traffic Model Parameters: free_flow_speed = {}, a = {}, e = {}, theta = {}, jam_density = {}",
                self.free_flow_speed, self.a, self.e, self.theta, self.jam_density
            );
        }

        /// convert the traffic model parameters to a vector
        /// params[0] = free_flow_speed
        /// params[1] = a
        /// params[2] = e
        /// params[3] = theta
        /// params[4] = jam_density
        fn get_params_as_vec(&self) -> Vec<f64> {
            vec![self.free_flow_speed, self.a, self.e, self.theta, self.jam_density]
        }

        /// create ModifiedLee from vector of parameters
        /// params[0] = free_flow_speed
        /// params[1] = a
        /// params[2] = e
        /// params[3] = theta
        /// params[4] = jam_density
        /// panics if params.len() != 5
        fn from_vec(params: &Vec<f64>) -> Self {
            assert!(params.len() == 5);
            Self {
                free_flow_speed: params[0],
                a: params[1],
                e: params[2],
                theta: params[3],
                jam_density: params[4],
                jam_density_min: MIN_JAM_DENSITY, // default value, not used in this context
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::traffic_model::{TrafficModel, modified_lee::ModifiedLee};

        #[test]
        fn test_modified_lee_speed_positive() {
            let model = ModifiedLee::new(13.6, 60.0);

            let speed = model.get_speed(2.0);
            dbg!(&speed);
            assert!(speed > 0.0);
        }

        #[test]
        fn test_free_flow_speed() {
            let model = ModifiedLee::new(13.6, 60.0);

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
                jam_density_min: 60.0,
                free_flow_speed: 109.0,
            };

            let speed = model.get_speed(45.0);
            dbg!(&speed);
            assert!(speed > 0.0);
        }
    }
}
