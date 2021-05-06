use lapin::{
    Connection, 
    ConnectionProperties
};
use tokio_amqp::{
    LapinTokioExt
};

#[tokio::main]
async fn main(){
    let rabbit_connection_properties = ConnectionProperties::default()
        .with_tokio();
    let _rabbit_conn = Connection::connect("amqp://127.0.0.1:5672/%2f", rabbit_connection_properties)
        .await
        .expect("Rabbit connection failed");
}
