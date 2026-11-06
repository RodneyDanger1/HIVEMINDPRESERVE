//! Identifier newtypes.
//!
//! Every identifier on the wire is a uuid-v7 string (time-ordered, per
//! `docs/contracts/UI-COMMANDS-v1.md` §1). Newtypes keep run/task/approval ids
//! from being accidentally mixed.

use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

macro_rules! id_newtype {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(Uuid);

        impl $name {
            /// Creates a fresh uuid-v7 identifier.
            #[must_use]
            pub fn new() -> Self {
                Self(Uuid::now_v7())
            }

            /// Wraps an identifier read back from storage.
            #[must_use]
            pub const fn from_uuid(id: Uuid) -> Self {
                Self(id)
            }

            /// Borrows the underlying uuid.
            #[must_use]
            pub const fn as_uuid(&self) -> Uuid {
                self.0
            }

            /// Consumes the newtype and returns the underlying uuid.
            #[must_use]
            pub const fn into_uuid(self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(&self.0, f)
            }
        }

        impl FromStr for $name {
            type Err = uuid::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(s.parse()?))
            }
        }

        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                self.0.serialize(serializer)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                Uuid::deserialize(deserializer).map(Self)
            }
        }

        impl JsonSchema for $name {
            fn schema_name() -> Cow<'static, str> {
                stringify!($name).into()
            }

            fn schema_id() -> Cow<'static, str> {
                concat!(module_path!(), "::", stringify!($name)).into()
            }

            fn json_schema(_generator: &mut SchemaGenerator) -> Schema {
                json_schema!({"type": "string", "format": "uuid"})
            }
        }
    };
}

id_newtype!(
    /// Identifies one run (one user prompt and everything it spawns).
    RunId
);
id_newtype!(
    /// Identifies one task (one node in the run graph).
    TaskId
);
id_newtype!(
    /// Identifies one approval request.
    ApprovalId
);

impl RunId {
    /// uuid-v7 timestamp in milliseconds since the Unix epoch, if the id is a
    /// v7 id. Used only for diagnostics ordering; never for correctness.
    #[must_use]
    pub fn unix_millis(&self) -> Option<u64> {
        self.0.get_timestamp().map(|ts| ts.to_unix().0)
    }
}
