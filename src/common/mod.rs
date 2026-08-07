//! Element definitions common to multiple EML_NL document variants.

mod candidate_identifier;
mod canonicalization_method;
mod committee;
mod contest_identifier;
mod country_name_code;
mod creation_date_time;
mod election_domain;
mod election_tree;
mod issue_date;
mod list_data;
mod locality_name;
mod managing_authority;
mod minimal_qualifying_address;
mod person_name;
mod postal_code;
mod region;
mod reporting_unit_identifier;
mod transaction_id;

pub use candidate_identifier::*;
pub use canonicalization_method::*;
pub use committee::*;
pub use contest_identifier::*;
pub use country_name_code::*;
pub use creation_date_time::*;
pub use election_domain::*;
pub use election_tree::*;
pub use issue_date::*;
pub use list_data::*;
pub use locality_name::*;
pub use managing_authority::*;
pub use minimal_qualifying_address::*;
pub use person_name::*;
pub use postal_code::*;
pub use region::*;
pub use reporting_unit_identifier::*;
pub use transaction_id::*;
