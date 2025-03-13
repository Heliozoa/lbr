//! Generic utilities for working with diesel.

pub use crate::{diesel_enum, diesel_struct, eq, query};
use std::slice::Chunks;

pub const PG_MAX_PARAMS: usize = 65535;

#[macro_export]
macro_rules! diesel_struct {
    (
        $(#[ $attr:meta ])*
        $t:ident {
            $($field:ident: $field_t:ty = $diesel_t:tt),* $(,)?
        }
    ) => {
        $(#[ $attr ])*
        #[derive(Debug, ::diesel::AsExpression, ::diesel::FromSqlRow)]
        #[diesel(sql_type = $crate::schema::sql_types::$t)]
        pub struct $t {
            $(pub $field: $field_t),*
        }

        impl ::diesel::serialize::ToSql<$crate::schema::sql_types::$t, ::diesel::pg::Pg> for $t {
            fn to_sql<'b>(
                    &'b self,
                    out: &mut ::diesel::serialize::Output<'b, '_, ::diesel::pg::Pg>
                ) -> ::diesel::serialize::Result {
                ::diesel::serialize::WriteTuple::<($(::diesel::sql_types::$diesel_t),*)>::write_tuple(
                    &($(self.$field),*),
                    out,
                )
            }
        }

        impl ::diesel::query_builder::QueryId for $crate::schema::sql_types::$t {
            type QueryId = $t;

            const HAS_STATIC_QUERY_ID: bool = true;
        }

        impl ::diesel::deserialize::FromSql<$crate::schema::sql_types::$t, ::diesel::pg::Pg> for $t {
            fn from_sql(
                bytes: <::diesel::pg::Pg as ::diesel::backend::Backend>::RawValue<'_>
            ) -> ::diesel::deserialize::Result<Self> {
                let ($($field),*) =
                    <($($field_t),*) as ::diesel::deserialize::FromSql<_, _>>::from_sql(bytes)?;
                Ok(Self {
                    $($field),*
                })
            }
        }
    };
}

#[macro_export]
macro_rules! diesel_enum {
    (
        $(#[ $attr:meta ])*
        $t:ident {
            $($variant:ident: $l:literal),*
        }
    ) => {
        diesel_enum!(
            #[$($attr),*]
            $t: $t {
                $($variant: $l),*
            }
        );
    };
    (
        $(#[ $attr:meta ])*
        $t:ident: $dt:ident {
            $($variant:ident: $l:literal),*
        }
    ) => {
        $(#[ $attr ])*
        #[derive(Debug, ::diesel::AsExpression, ::diesel::FromSqlRow)]
        #[diesel(sql_type = $crate::schema::sql_types::$dt)]
        pub enum $t {
            $($variant),*
        }

        impl ::diesel::serialize::ToSql<$crate::schema::sql_types::$dt, ::diesel::pg::Pg> for $t {
            fn to_sql<'b>(&'b self, out: &mut ::diesel::serialize::Output<'b, '_, ::diesel::pg::Pg>) -> ::diesel::serialize::Result {
                match self {
                    $(
                        Self::$variant => <str as ::diesel::serialize::ToSql<::diesel::sql_types::Text, ::diesel::pg::Pg>>::to_sql($l, out),
                    )*
                }
            }
        }

        impl ::diesel::query_builder::QueryId for $crate::schema::sql_types::$dt {
            type QueryId = $t;

            const HAS_STATIC_QUERY_ID: bool = true;
        }

        impl ::diesel::deserialize::FromSql<$crate::schema::sql_types::$dt, ::diesel::pg::Pg> for $t {
            fn from_sql(
                bytes: <::diesel::pg::Pg as ::diesel::backend::Backend>::RawValue<'_>
            ) -> ::diesel::deserialize::Result<Self> {
                let variant_name =
                    <String as ::diesel::deserialize::FromSql<::diesel::sql_types::Text, ::diesel::pg::Pg>>::from_sql(bytes)?;
                let res = match variant_name.as_str() {
                    $($l => Self::$variant,)*
                    other => panic!("Invalid data from database: {other}"),
                };
                Ok(res)
            }
        }
    };
}

/// Helper macro for making queries.
///
/// eq!(table, column_1, column_2)
/// =
/// (table::column_1.eq(column_1), table::column_2.eq(column_2))
///
/// eq!(table_1::column_1, table_2::column_2)
/// =
/// (table_1::column_1.eq(column_1), table_2::column_2.eq(column_2))
#[macro_export]
macro_rules! eq {
    ($t:ident, $c: ident $(,)?) => {
        $t::$c.eq($c)
    };
    ($t:ident, $($c: ident),* $(,)?) => {
        ( $($t::$c.eq($c)),* )
    };
    ($t:ident :: $c: ident) => {
        $t::$c.eq($c)
    };
    ($($t:ident :: $c: ident),* $(,)?) => {
        ( $($t::$c.eq($c)),* )
    };
}

/// Helper macro for implementing Queryable and Selectable and ensures the implementations match.
///
/// ```
/// query! {
///     #[derive(Debug, Serialize)]
///     pub struct PostSmall {
///         pub id: i32 = posts::id,
///         pub thumbnail: String = posts::thumbnail_filename,
///         pub title: String = posts::title,
///         pub user_id: i32 = users::id,
///         pub username: String = users::display_name,
///     }
/// }
/// ```
#[macro_export]
macro_rules! query {
    (
        $(#[ $attr:meta ])*
        $v:vis $kw:ident $name:ident {
            $(
                $fv:vis $field:ident: $t:ty = $table:ident :: $column:ident
            ),* $(,)?
        }
    ) => {
        $(#[ $attr ])*
        #[derive(::diesel::Queryable)]
        #[diesel(check_for_backend(::diesel::pg::Pg))]
        $v $kw $name {
            $($fv $field: $t),*
        }

        impl<DB: ::diesel::backend::Backend> ::diesel::Selectable<DB> for $name {
            type SelectExpression = ($( $crate::schema::$table::$column, )*);

            fn construct_selection() -> Self::SelectExpression {
                ($( $crate::schema::$table::$column, )*)
            }
        }
    };
}

/// Helper macro for implementing Queryable and Selectable and ensures the implementations match.
///
/// ```
/// query! {
///     #[derive(Debug, Serialize)]
///     pub struct PostSmall {
///         pub id: i32 = posts::id,
///         pub thumbnail: String = posts::thumbnail_filename,
///         pub title: String = posts::title,
///         pub user_id: i32 = users::id,
///         pub username: String = users::display_name,
///     }
/// }
/// ```
#[macro_export]
macro_rules! newquery {
    (
        $(#[ $attr:meta ])*
        $v:vis $kw:ident $name:ident {
            $(
                $fv:vis $field:ident: $t:ty = $selection:expr
            ),* $(,)?
        }
    ) => {
        $(#[ $attr ])*
        #[derive(::diesel::Queryable)]
        #[derive(::diesel::Selectable)]
        #[diesel(table_name = $crate::schema::users)]
        #[diesel(check_for_backend(::diesel::pg::Pg))]
        $v $kw $name {
            $(
                #[diesel(select_expression = $crate::schema :: $selection)]
                $fv $field: $t
            ),*
        }
    };
}

pub trait PostgresChunks<T> {
    fn pg_chunks(&self) -> Chunks<'_, T>;
}

macro_rules! impl_postgres_chunks {
    (
        $lit:literal, $($ty:ident),*
    ) => {
        impl<$($ty),*,> PostgresChunks<($($ty),*,)> for Vec<($($ty),*,)> {
            fn pg_chunks(&self) -> Chunks<'_, ($($ty),*,)> {
                self.chunks(PG_MAX_PARAMS / $lit)
            }
        }
    };
}

impl_postgres_chunks!(2, A, B);
impl_postgres_chunks!(3, A, B, C);
impl_postgres_chunks!(4, A, B, C, D);
impl_postgres_chunks!(5, A, B, C, D, E);
impl_postgres_chunks!(6, A, B, C, D, E, F);
impl_postgres_chunks!(7, A, B, C, D, E, F, G);
impl_postgres_chunks!(8, A, B, C, D, E, F, G, H);
