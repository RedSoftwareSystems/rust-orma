use orma::DbData;
use orma_derive::orma_obj;
use serde::{Deserialize, Serialize};

#[test]
fn proc_macro_attr_table_attr() {
    #[orma_obj(table = "xxx")]
    #[derive(Serialize, Deserialize)]
    struct Foo {
        name: String,
    }
    assert_eq!(Foo::table_name(), "xxx");
}

fn data(input: &str) -> &str {
    input
}

#[orma_derive::test]
async fn test_orma_test(data: &str) {
    assert_eq!(data, "test_orma_test");
}
