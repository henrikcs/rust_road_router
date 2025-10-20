pub struct Logger {
    application: String,
    identifier: String,
    iteration: i32,
}

impl Logger {
    pub fn new(application: &str, identifier: &str, iteration: i32) -> Self {
        Logger {
            application: application.to_string(),
            identifier: identifier.to_string(),
            iteration,
        }
    }

    /// Logs the operation with the duration in nanoseconds within a certain iteration of certain run identified by identifier.
    /// The format is: "sumo-fastdta-router; <identifier>; <iteration>; <operation>; <duration_in_nanos>"
    pub fn log(&self, operation: &str, duration_in_nanos: u128) {
        println!(
            "{}; {}; {}; {}; {}",
            self.application, self.identifier, self.iteration, operation, duration_in_nanos
        );
    }
}
