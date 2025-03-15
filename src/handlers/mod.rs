mod init;
mod process;
mod result;
mod runner;

pub use init::{InitResult, initialize_app};
pub use process::spawn_processing_thread;
pub use result::handle_organization_result;
pub use runner::run_app;
