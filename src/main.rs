mod app;
mod audio;
mod config;
mod constants;
mod error;
mod smoothing;
mod state;
mod ui;

use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait};
use dialoguer::{Select, theme::ColorfulTheme};

fn list_devices() -> Result<(), Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let devices = host.input_devices()?;

    let device_list: Vec<String> = devices.filter_map(|d| d.name().ok()).collect();

    if device_list.is_empty() {
        println!("No audio input devices found.");
        return Ok(());
    }

    // Interactive selection
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select an audio input device")
        .items(&device_list)
        .default(0)
        .interact()?;

    println!("{}", device_list[selection]);

    Ok(())
}

#[tokio::main]
async fn main() {
    use app::ExitCode;
    use config::{Args, Commands};

    let args = Args::parse();

    match args.command {
        Commands::Detect(detect_args) => {
            // Create config from detect args
            let config = match config::Config::from_detect_args(detect_args) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Configuration error: {}", e);
                    std::process::exit(app::ExitCode::Error as i32);
                }
            };

            // Handle exit codes appropriately
            match app::App::new_with_config(config) {
                Ok(app) => {
                    let run_result = app.run().await;
                    match run_result.result {
                        Ok(_) => {
                            std::process::exit(run_result.exit_code as i32);
                        }
                        Err(e) => {
                            eprintln!("Application error: {}", e);
                            std::process::exit(ExitCode::Error as i32);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Setup error: {}", e);
                    std::process::exit(ExitCode::Error as i32);
                }
            }
        }
        Commands::List(_) => {
            if let Err(e) = list_devices() {
                eprintln!("Error listing devices: {}", e);
                std::process::exit(app::ExitCode::Error as i32);
            }
        }
        Commands::Max(max_args) => {
            // Create config from max args
            let config = match config::Config::from_max_args(&max_args) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Configuration error: {}", e);
                    std::process::exit(app::ExitCode::Error as i32);
                }
            };

            // Create app
            let mut app = match app::App::new_with_config(config) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("Setup error: {}", e);
                    std::process::exit(app::ExitCode::Error as i32);
                }
            };

            // Run max monitoring
            match app.run_max(max_args.seconds).await {
                Ok(max_levels) => {
                    if max_args.quiet {
                        for &level in &max_levels {
                            println!("{}", level.round() as i32);
                        }
                    } else {
                        println!("Maximum dB levels detected:");
                        for (i, &level) in max_levels.iter().enumerate() {
                            println!("Channel {}: {:.1} dB", i, level);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error during monitoring: {}", e);
                    std::process::exit(app::ExitCode::Error as i32);
                }
            }
        }
        Commands::Average(average_args) => {
            // Create config from average args
            let config = match config::Config::from_average_args(&average_args) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Configuration error: {}", e);
                    std::process::exit(app::ExitCode::Error as i32);
                }
            };

            // Create app
            let mut app = match app::App::new_with_config(config) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("Setup error: {}", e);
                    std::process::exit(app::ExitCode::Error as i32);
                }
            };

            // Run average monitoring
            match app.run_average(average_args.seconds).await {
                Ok(avg_levels) => {
                    if average_args.quiet {
                        for &level in &avg_levels {
                            println!("{}", level.round() as i32);
                        }
                    } else {
                        println!("Average dB levels detected:");
                        for (i, &level) in avg_levels.iter().enumerate() {
                            println!("Channel {}: {:.1} dB", i, level);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error during monitoring: {}", e);
                    std::process::exit(app::ExitCode::Error as i32);
                }
            }
        }
    }
}
