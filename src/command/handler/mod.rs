pub(crate) mod uds_client;
pub(crate) mod user_input;

use std::error::Error;
use std::result;
type HandleResult = result::Result<(), Box<dyn Error>>;
