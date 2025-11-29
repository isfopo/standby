mod app;
mod audio;
mod config;
mod constants;
mod error;
mod smoothing;
mod state;
mod ui;


#[tokio::main]
async fn main() {
    use app::ExitCode;

    // Handle exit codes appropriately
    match app::App::new() {
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


