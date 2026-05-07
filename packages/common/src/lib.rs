pub mod rut;
pub mod attendance;
pub mod grades;
pub mod student;
pub mod user;
pub mod academic;
pub mod communication;
pub mod finance;
pub mod reporting;
pub mod modules;

#[cfg(feature = "db")]
pub mod db_schema;
