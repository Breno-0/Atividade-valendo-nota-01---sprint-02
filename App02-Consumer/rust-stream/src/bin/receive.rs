use futures_util::stream::StreamExt;
use lapin::{
    options::{ BasicAckOptions, BasicConsumeOptions, QueueDeclareOptions },
    types::FieldTable,
    Connection,
    ConnectionProperties,
};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::connect(
        "amqp://127.0.0.1:5672/%2f",
        ConnectionProperties::default()
    ).await?;

    println!("Conectado ao RabbitMQ!");

    let channel = connection.create_channel().await?;

    channel.queue_declare(
        "validacao_usuarios",
        QueueDeclareOptions {
            durable: true,
            ..Default::default()
        },
        FieldTable::default()
    ).await?;

    let mut consumer = channel.basic_consume(
        "validacao_usuarios",
        "rust-consumer",
        BasicConsumeOptions::default(),
        FieldTable::default()
    ).await?;

    println!("Aguardando mensagens...");

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery?;

        let message = String::from_utf8(delivery.data.clone())?;

        println!("Mensagem recebida: {}", message);

        delivery.ack(BasicAckOptions::default()).await?;
    }

    Ok(())
}
