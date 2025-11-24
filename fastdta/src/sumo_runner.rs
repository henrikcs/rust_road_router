use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Configuration for running SUMO simulation
pub struct SumoConfig {
    pub net_file: PathBuf,
    pub routes_file: PathBuf,
    pub additional_file: PathBuf,
    pub precision: u32,
    pub step_length: f64,
}

impl SumoConfig {
    pub fn new(net_file: PathBuf, routes_file: PathBuf, additional_file: PathBuf) -> Self {
        Self {
            net_file,
            routes_file,
            additional_file,
            precision: 6,
            step_length: 0.1,
        }
    }
}

/// Run SUMO simulation with the specified configuration
/// Returns Ok(()) if the simulation was successful, otherwise returns an error
pub fn run_sumo(config: &SumoConfig) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("sumo")
        .arg("-n")
        .arg(&config.net_file)
        .arg("--routes")
        .arg(&config.routes_file)
        .arg("-a")
        .arg(&config.additional_file)
        .arg("--precision")
        .arg(config.precision.to_string())
        .arg("--step-length")
        .arg(config.step_length.to_string())
        .arg("--mesosim")
        .arg("--ignore-route-errors")
        .arg("--aggregate-warnings")
        .arg("5")
        .arg("--time-to-teleport.disconnected")
        .arg("0")
        .status()?;

    if !status.success() {
        return Err(Box::from(format!("SUMO simulation failed with exit code: {}", status.code().unwrap_or(-1))));
    }

    Ok(())
}

/// Generate SUMO additional file for edgeData output
/// Format: <a><edgeData id="dump_<aggregation>" freq="<aggregation>" file="dump_<aggregation>_<iteration>_<batch>.xml" excludeEmpty="true" minSamples="1"/></a>
pub fn generate_additional_file(output_path: &Path, aggregation: u32, iteration: u32, batch: u32) -> Result<(), Box<dyn std::error::Error>> {
    let dump_filename = format!("dump_{}_{:0>3}_{:0>3}.xml", aggregation, iteration, batch);
    let dump_path = output_path.parent().unwrap().join(&dump_filename);

    let content = format!(
        r#"<a>
    <edgeData id="dump_{}" freq="{}" file="{}" excludeEmpty="true" minSamples="1"/>
</a>"#,
        aggregation,
        aggregation,
        dump_path.display()
    );

    let mut file = std::fs::File::create(output_path)?;
    file.write_all(content.as_bytes())?;

    Ok(())
}
