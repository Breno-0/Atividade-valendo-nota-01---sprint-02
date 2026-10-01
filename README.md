# Integração de aplicações com RabbitMQ

Atividade prática da Sprint 02: duas aplicações independentes que se comunicam de forma assíncrona por meio do RabbitMQ, executadas com Docker Compose.

## Integrantes

| Nome completo | RA |
| --- | --- |
| `Breno Otávio Silva Costa` | `04251072` |
| `Matheus Nascimento Torres de Souza` | `04251033` |

## Visão geral da solução

```
App01 (Python)                  RabbitMQ                     App02 (Rust)
     |                     fila: validacao_usuarios               |
     |  POST /usuarios            |                               |
     |----> Producer ------------>|------------------------------>|
     |      valida campos         |   mensagem JSON persistente   | 
     |      vazios e nulos        |                               |
     |                            |                               | valida regras de negócio
     |<---- 201 / 400             |                               | e imprime o resultado no log
```

| Serviço | Tecnologia | Papel |
| --- | --- | --- |
| `rabbitmq` | RabbitMQ 4 (imagem com painel de gerenciamento) | Broker de mensagens |
| `app01-producer` | Python 3.11, Flask e pika | API HTTP que recebe o cadastro de um usuário, confere se todos os campos foram enviados e publica a mensagem na fila `validacao_usuarios` |
| `app02-consumer` | Rust, lapin e tokio | Consome a fila `validacao_usuarios`, valida os dados do usuário e imprime no log se ele é válido ou quais regras falharam |

Nenhuma das aplicações precisa de banco de dados.

### Regras validadas pelo consumer (App02)

- Nome não pode ser vazio.
- Email não pode ser vazio e precisa conter `@` e `.`.
- Senha precisa ter pelo menos 8 caracteres, pelo menos um número e pelo menos um caractere especial.
- Data de nascimento precisa estar no formato `DD/MM/AAAA`, com dia, mês e ano válidos (ano entre 1900 e 2026, considerando anos bissextos).

## Estrutura do projeto

```
.
├── App01-Producer/
│   ├── app.py
│   ├── requirements.txt
│   └── Dockerfile
├── App02-Consumer/
│   ├── rust-amqp/
│   │   ├── src/
│   │   ├── Cargo.toml
│   │   └── Cargo.lock
│   ├── .dockerignore
│   └── Dockerfile
├── compose.yml
└── README.md
```

## Pré-requisitos

- Docker com Docker Compose v2 (comando `docker compose`).
- Portas `5000` (API do App01) e `15672` (painel do RabbitMQ) livres na máquina.
- [Bruno](https://www.usebruno.com/) para enviar as requisições HTTP ao App01.

## Como executar

Na pasta raiz do projeto (onde está o `compose.yml`):

```bash
docker compose up --build
```

Esse comando constrói as imagens das duas aplicações e sobe RabbitMQ, App01 e App02. As aplicações só iniciam depois que o RabbitMQ estiver pronto para receber conexões.

A solução está pronta quando aparecer no log:

```
app02-consumer  | Conectado ao RabbitMQ!
app02-consumer  | Aguardando mensagens...
```

## Como testar a comunicação

### 1. Acompanhar o log do consumer

Os logs dos três serviços aparecem misturados no terminal do `docker compose up`. Para ver apenas o que o App02 processa, abra um segundo terminal na pasta do projeto e execute:

```bash
docker compose logs -f app02-consumer
```

### 2. Configurar a requisição no Bruno

As requisições ao App01 são feitas pelo [Bruno](https://www.usebruno.com/). Abra uma coleção (ou crie uma nova) e adicione uma requisição HTTP com:

- **Método:** `POST`
- **URL:** `http://localhost:5000/usuarios`
- **Body:** aba **Body**, opção **JSON**

Ao escolher JSON no Body, o Bruno já envia o cabeçalho `Content-Type: application/json`. A mesma requisição serve para todos os testes abaixo: basta trocar o JSON do Body e enviar com **Ctrl+Enter** (ou pelo botão de envio).

### 3. Enviar um usuário válido

Body:

```json
{
  "nome": "Ana Souza",
  "email": "ana@email.com",
  "senha": "Senha@123",
  "data_nascimento": "15/08/2000"
}
```

Resposta esperada no Bruno: status `201` e o corpo

```json
{
  "mensagem": "Usuário Ana Souza enviado para a fila! DEU BOM!"
}
```

Saída esperada no log do App02:

```
app02-consumer  | Nome: Ana Souza
app02-consumer  | Email: ana@email.com
app02-consumer  | Data de nascimento: 15/08/2000
app02-consumer  | Usuário válido!
```

### 4. Enviar um usuário com dados inválidos

O App01 aceita a mensagem, porque todos os campos foram preenchidos, e o App02 aponta as regras que falharam.

Body:

```json
{
  "nome": "Teste",
  "email": "teste.com",
  "senha": "abc",
  "data_nascimento": "31/02/2000"
}
```

Resposta esperada no Bruno: status `201`, porque a validação dessas regras acontece no App02.

Saída esperada no log do App02:

```
app02-consumer  | Nome: Teste
app02-consumer  | Email: teste.com
app02-consumer  | Data de nascimento: 31/02/2000
app02-consumer  | Usuário inválido:
app02-consumer  |  - Email inválido
app02-consumer  |  - Senha deve possuir pelo menos 8 caracteres
app02-consumer  |  - Senha deve possuir pelo menos um número
app02-consumer  |  - Senha deve possuir pelo menos um caractere especial
app02-consumer  |  - Dia de nascimento inválido
```

### 5. Enviar um cadastro com campo faltando

Quando algum campo obrigatório falta ou está vazio, o próprio App01 recusa a requisição e nada é publicado na fila.

Body (sem o campo `senha`):

```json
{
  "nome": "Sem Senha",
  "email": "sem@senha.com",
  "data_nascimento": "01/01/2000"
}
```

Resposta esperada no Bruno: status `400` e o corpo

```json
{
  "ERRO": "O campo \"senha\" é obrigatório e não pode ser vazio ou nulo."
}
```

Nada aparece no log do App02.

## Encerrando

```bash
docker compose down
```

## Solução de problemas

- **Linhas do log somem ou ficam embaralhadas no terminal:** o menu interativo do `docker compose up` pode sobrescrever linhas em alguns terminais (por exemplo, Git Bash no Windows). Use `docker compose up --build --menu=false` ou acompanhe o consumer com `docker compose logs -f app02-consumer`.
- **Erro de porta já em uso:** algum outro serviço está usando a porta `5000` ou `15672`. Encerre esse serviço ou altere a porta do lado esquerdo do mapeamento no `compose.yml` (por exemplo, `"5001:5000"`).
