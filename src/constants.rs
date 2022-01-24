

//pub const REDIS_CONFIG: &'static str = option_env!("REDIS_URL").unwrap_or("redis://127.0.0.1/");

//pub const COUCHDB_CONFIG: &'static str = env!("COUCHDB_URL").unwrap_or("http://127.0.0.1:5984/");

pub const JWT_SECRET: &'static [u8] =  
("NTNv7j0TuYARvmNMmWXo6fKvM4o6nv/aUi9ryX38ZH+L1bkrnD1ObOQ8JAUmHCBq7Iy7otZcyAagBLHVKvvYaIpmMuxmARQ97jUVG16Jkpkp1wXOPsrF9zwew6TpczyHkHgX5EuLg2MeBuiT/qJACs1J0apruOOJCg/gOtkjB4c=")
.as_bytes();

pub const BEARER: &'static str  = "Bearer ";