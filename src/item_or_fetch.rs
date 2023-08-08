use std::ops::Index;

pub trait ItemOrFetch<T> {
    fn item_or_fetch<'api>(&'api self, components: &'api Option<openapiv3::Components>) -> &T;
}

macro_rules! item_or_fetch_impl {
    ($item_ty:ty, $reference_ty:ty, $component_field:ident, $component_path:expr) => {
        impl ItemOrFetch<$item_ty> for $reference_ty {
            fn item_or_fetch<'api>(
                &'api self,
                components: &'api Option<openapiv3::Components>,
            ) -> &$item_ty {
                match self {
                    Self::Item(item) => item,
                    Self::Reference { reference } => components
                        .as_ref()
                        .unwrap()
                        .$component_field
                        .index(reference.trim_start_matches($component_path))
                        .item_or_fetch(components),
                }
            }
        }
    };
}

item_or_fetch_impl!(
    openapiv3::Schema,
    openapiv3::ReferenceOr<openapiv3::Schema>,
    schemas,
    "#/components/schemas/"
);
item_or_fetch_impl!(
    openapiv3::Parameter,
    openapiv3::ReferenceOr<openapiv3::Parameter>,
    parameters,
    "#/components/parameters/"
);
item_or_fetch_impl!(
    openapiv3::RequestBody,
    openapiv3::ReferenceOr<openapiv3::RequestBody>,
    request_bodies,
    "#/components/requestBodies/"
);
