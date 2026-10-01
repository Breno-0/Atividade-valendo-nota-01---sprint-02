import os
import json
import pika
from flask import Flask, request, jsonify

app = Flask(__name__)

RABBITMQ_HOST = os.getenv('RABBITMQ_HOST', 'localhost')

@app.route('/usuarios', methods=['POST'])
def cadastrar_usuario():

    dados = request.get_json()

    if not dados:
        return jsonify(
            {'ERRO': 'JSON inválido ou ausente'}
            ), 400

    campos_obrigatorios = ['nome', 'email', 'senha', 'data_nascimento']

    for campo in campos_obrigatorios:
        valor = dados.get(campo)
        
        if valor is None or (isinstance(valor, str) and not valor.strip()):
            return jsonify(
                {'ERRO': f'O campo "{campo}" é obrigatório e não pode ser vazio ou nulo.'}
                ), 400

    try:
        
        connection = pika.BlockingConnection(
            pika.ConnectionParameters(host=RABBITMQ_HOST)
        )
        
        channel = connection.channel()

        channel.queue_declare(queue='validacao_usuarios', durable=True)

        mensagem = json.dumps(dados)

        channel.basic_publish(
            exchange='',
            routing_key='validacao_usuarios',
            body=mensagem,
            properties=pika.BasicProperties(
                delivery_mode=pika.DeliveryMode.Persistent 
            )
        )

        connection.close()

        return jsonify({
            'mensagem': f'Usuário {dados["nome"]} enviado para a fila! DEU BOM!'
        }), 201

    except Exception as e:
        return jsonify({'ERRO': f'Erro ao conectar no RabbitMQ: {str(e)}'}), 500

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)