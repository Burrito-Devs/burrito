
macro_rules! make_wrapped_id_type {
    ($($name:ident),*) => {
        use serde_derive::{Deserialize, Serialize};
        $(
            #[derive(Clone, Copy, Debug, Eq, Hash, Default, Deserialize, Ord, PartialEq, PartialOrd, Serialize)]
            pub struct $name(pub u32);

            impl From<$name> for u32 {
                fn from(id: $name) -> u32 {
                    id.0
                }
            }

            impl From<u32> for $name {
                fn from(value: u32) -> Self {
                    $name(value)
                }
            }

            impl std::fmt::Display for $name {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(f, "{}", self.0)
                }
            }
        )*
    };
}

make_wrapped_id_type!(ConstellationId, RegionId, StargateId, StarId, SystemId);
