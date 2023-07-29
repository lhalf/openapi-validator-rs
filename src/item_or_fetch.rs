use std::ops::Index;

pub trait ItemOrFetch<T> {
    fn item_or_fetch<'api>(&'api self, components: &'api Option<openapiv3::Components>) -> &T;
}

impl ItemOrFetch<openapiv3::Schema> for openapiv3::ReferenceOr<openapiv3::Schema> {
    fn item_or_fetch<'api>(
        &'api self,
        components: &'api Option<openapiv3::Components>,
    ) -> &openapiv3::Schema {
        match self {
            Self::Item(item) => item,
            Self::Reference { reference } => components
                .as_ref()
                .unwrap()
                .schemas
                .index(reference.trim_start_matches("#/components/schemas/"))
                .item_or_fetch(components),
        }
    }
}

impl ItemOrFetch<openapiv3::Parameter> for openapiv3::ReferenceOr<openapiv3::Parameter> {
    fn item_or_fetch<'api>(
        &'api self,
        components: &'api Option<openapiv3::Components>,
    ) -> &openapiv3::Parameter {
        match self {
            Self::Item(item) => item,
            Self::Reference { reference } => components
                .as_ref()
                .unwrap()
                .parameters
                .index(reference.trim_start_matches("#/components/parameters/"))
                .item_or_fetch(components),
        }
    }
}