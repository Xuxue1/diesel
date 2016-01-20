use std::marker::PhantomData;

use expression::Expression;
use query_builder::{QueryBuilder, BuildQueryResult, QueryFragment};
use query_source::{Table, Column};
use types::NativeSqlType;

/// Represents that a structure can be used to to insert a new row into the database.
/// Implementations can be automatically generated by
/// [`#[insertable_into]`](https://github.com/sgrif/diesel/tree/master/diesel_codegen#insertable_intotable_name).
/// This is automatically implemented for `&[T]`, `Vec<T>` and `&Vec<T>` for inserting more than
/// one record.
pub trait Insertable<T: Table> {
    type Columns: InsertableColumns<T>;
    type Values: Expression<SqlType=<Self::Columns as InsertableColumns<T>>::SqlType> + QueryFragment;

    fn columns() -> Self::Columns;

    fn values(self) -> Self::Values;
}

pub trait InsertableColumns<T: Table> {
    type SqlType: NativeSqlType;

    fn names(&self) -> String;
}

impl<'a, T, U> Insertable<T> for &'a [U] where
    T: Table,
    &'a U: Insertable<T>,
{
    type Columns = <&'a U as Insertable<T>>::Columns;
    type Values = InsertValues<'a, T, U>;

    fn columns() -> Self::Columns {
        <&'a U>::columns()
    }

    fn values(self) -> Self::Values {
        InsertValues {
            values: self,
            _marker: PhantomData,
        }
    }
}

impl<'a, T, U> Insertable<T> for &'a Vec<U> where
    T: Table,
    &'a U: Insertable<T>,
{
    type Columns = <&'a U as Insertable<T>>::Columns;
    type Values = InsertValues<'a, T, U>;

    fn columns() -> Self::Columns {
        <&'a U>::columns()
    }

    fn values(self) -> Self::Values {
        InsertValues {
            values: &*self,
            _marker: PhantomData,
        }
    }
}


pub struct InsertValues<'a, T, U: 'a> {
    values: &'a [U],
    _marker: PhantomData<T>,
}

impl<'a, T, U> Expression for InsertValues<'a, T, U> where
    T: Table,
    &'a U: Insertable<T>,
{
    type SqlType = <<&'a U as Insertable<T>>::Columns as InsertableColumns<T>>::SqlType;
}

impl<'a, T, U> QueryFragment for InsertValues<'a, T, U> where
    T: Table,
    &'a U: Insertable<T>,
{
    fn to_sql(&self, out: &mut QueryBuilder) -> BuildQueryResult {
        for (i, record) in self.values.into_iter().enumerate() {
            if i != 0 {
                out.push_sql(", ");
            }
            try!(record.values().to_sql(out));
        }
        Ok(())
    }
}

impl<C: Column<Table=T>, T: Table> InsertableColumns<T> for C {
    type SqlType = <Self as Expression>::SqlType;

    fn names(&self) -> String {
        Self::name().to_string()
    }
}
