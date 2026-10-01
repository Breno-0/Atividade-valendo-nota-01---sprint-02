use futures_util::stream::StreamExt;
use lapin::{
    options::{ BasicAckOptions, BasicConsumeOptions, QueueDeclareOptions },
    types::FieldTable,
    Connection,
    ConnectionProperties,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Usuario {
    nome: String,
    email: String,
    senha: String,
    data_nascimento: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rabbitmq_host = std::env::var("RABBITMQ_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let amqp_addr = format!("amqp://{}:5672/%2f", rabbitmq_host);

    let connection = Connection::connect(
        &amqp_addr,
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

        let usuario: Usuario = match serde_json::from_str(&message) {
            Ok(usuario) => usuario,
            Err(erro) => {
                println!("JSON inválido: {}", erro);
                delivery.ack(BasicAckOptions::default()).await?;
                continue;
            }
        };
        println!("Nome: {}", usuario.nome);
        println!("Email: {}", usuario.email);
        println!("Data de nascimento: {}", usuario.data_nascimento);

        match validar_usuario(&usuario) {
            Ok(()) => {
                println!("Usuário válido!");
            }

            Err(erros) => {
                println!("Usuário inválido:");

                for erro in erros {
                    println!(" - {}", erro);
                }
            }
        }

        delivery.ack(BasicAckOptions::default()).await?;
    }

    Ok(())
}

fn validar_usuario(usuario: &Usuario) -> Result<(), Vec<String>> {
    let mut erros = Vec::new();

    if usuario.nome.trim().is_empty() {
        erros.push("Nome não pode ser vazio".to_string());
    }

    if usuario.email.trim().is_empty() {
        erros.push("Email não pode ser vazio".to_string());
    }

    if !usuario.email.contains('@') {
        erros.push("Email inválido".to_string());
    }

    if !usuario.email.contains('.') {
        erros.push("Email inválido".to_string());
    }

    if usuario.senha.len() < 8 {
        erros.push("Senha deve possuir pelo menos 8 caracteres".to_string());
    }

    if !usuario.senha.chars().any(|c| c.is_ascii_digit()) {
        erros.push("Senha deve possuir pelo menos um número".to_string());
    }

    if !usuario.senha.chars().any(|c| !c.is_alphanumeric()) {
        erros.push("Senha deve possuir pelo menos um caractere especial".to_string());
    }

    if usuario.data_nascimento.len() != 10 {
        erros.push("Data de nascimento deve estar no formato DD/MM/AAAA".to_string());
    } else {
        let partes: Vec<&str> = usuario.data_nascimento.split('/').collect();

        if
            partes.len() != 3 ||
            partes[0].len() != 2 ||
            partes[1].len() != 2 ||
            partes[2].len() != 4 ||
            !partes[0].chars().all(|c| c.is_ascii_digit()) ||
            !partes[1].chars().all(|c| c.is_ascii_digit()) ||
            !partes[2].chars().all(|c| c.is_ascii_digit())
        {
            erros.push("Data de nascimento deve estar no formato DD/MM/AAAA".to_string());
        } else {
            let dia: u32 = partes[0].parse().unwrap();
            let mes: u32 = partes[1].parse().unwrap();
            let ano: u32 = partes[2].parse().unwrap();

            if mes < 1 || mes > 12 {
                erros.push("Mês de nascimento inválido".to_string());
            } else {
                let dias_no_mes = match mes {
                    2 if ano % 400 == 0 || (ano % 4 == 0 && ano % 100 != 0) => 29,
                    2 => 28,
                    4 | 6 | 9 | 11 => 30,
                    _ => 31,
                };

                if dia < 1 || dia > dias_no_mes {
                    erros.push("Dia de nascimento inválido".to_string());
                }
            }

            if ano < 1900 || ano > 2026 {
                erros.push("Ano de nascimento inválido".to_string());
            }
        }
    }

    if erros.is_empty() {
        Ok(())
    } else {
        Err(erros)
    }
}
