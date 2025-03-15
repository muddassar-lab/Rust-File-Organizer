mod init;
mod process;
mod result;

pub use init::{InitResult, initialize_app};
pub use process::spawn_processing_thread;
pub use result::handle_organization_result;
