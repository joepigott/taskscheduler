use std::env::var;
use std::net::SocketAddr;
use std::str::FromStr;

const SERV_ADDRESS: &'static str = "SERVER_ADDR";
const SCHED_TIMEOUT: &'static str = "SCHEDULER_TIMEOUT";
const SERV_TIMEOUT: &'static str = "SERVER_TIMEOUT";

/// Fetches the server address environment variable as a `SocketAddr`.
pub fn server_address() -> Result<SocketAddr, String> {
    match var(SERV_ADDRESS) {
        Ok(address) => {
            Ok(SocketAddr::from_str(&address).map_err(|_| format!("{SERV_ADDRESS} environment variable is invalid"))?)
        }
        Err(_) => {
            Err(format!("{SERV_ADDRESS} environment variable does not exist"))
        }
    }
}

/// Fetches the scheduler timeout environment variable (in milliseconds) as a 
/// `usize`. This controls how frequently the scheduler will apply scheduling
/// logic. It also controls how fast the scheduler thread will exit on average 
/// as a consequence.
pub fn scheduler_timeout() -> Result<usize, String> {
    match var(SCHED_TIMEOUT) {
        Ok(timeout) => {
            Ok(usize::from_str(&timeout).map_err(|_| format!("{SCHED_TIMEOUT} environment variable is invalid"))?)
        }
        Err(_) => {
            Err(format!("{SCHED_TIMEOUT} environment variable does not exist"))
        }
    }
}

/// Fetches the server timeout environment variable (in milliseconds) as a 
/// `usize`. This controls how fast the server thread will exit on average.
pub fn server_timeout() -> Result<usize, String> {
    match var(SERV_TIMEOUT) {
        Ok(timeout) => {
            Ok(usize::from_str(&timeout).map_err(|_| format!("{SERV_TIMEOUT} environment variable is invalid"))?)
        }
        Err(_) => {
            Err(format!("{SERV_TIMEOUT} environment variable does not exist"))
        }
    }
}
