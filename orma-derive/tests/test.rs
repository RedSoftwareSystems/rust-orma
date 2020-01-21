use orma_derive::DbData;

use serde_derive::Serialize;

use orma::*;

#[test]
fn test_derive() {
    #[pk(field_1 = "field1", field_2 = "field2")]
    #[table("schema.table_name")]
    #[derive(Serialize, DbData)]
    struct TestData {
        field_1: String,
        field_2: String,
        some_other_filed: String,
    }
    assert_eq!(TestData::table_name(), "schema.table_name");
}
