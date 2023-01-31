use mongodb::{Client, options::IndexOptions, IndexModel, bson::doc};
use crate::DB_NAME;


pub async fn create_generic_index<T>(client: &Client, field: String, collection: String){
    
    let options = IndexOptions::builder().unique(true).build();
    let model = IndexModel::builder()
    .keys(doc!{field:1})
    .options(options)
    .build();

    client.database(DB_NAME)
    .collection::<T>(&collection)
    .create_index(model,None)
    .await
    .expect("creatring an index should succeed");
}