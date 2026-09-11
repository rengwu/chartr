"""Local, deterministic OpenAI-compatible responder. Never calls a model service."""
import json
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


class Handler(BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def do_POST(self):
        request = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
        if '/responses' in self.path:
            return self.respond_codex(request)
        if '/messages' in self.path:
            return self.anthropic(request)
        self.send_response(200)
        self.send_header('Content-Type', 'text/event-stream')
        self.send_header('Connection', 'close')
        self.end_headers()
        messages = request.get('messages', [])
        last_user = next((i for i in range(len(messages)-1, -1, -1) if messages[i].get('role') == 'user'), -1)
        should_ask = last_user >= 0 and 'Ask a question' in str(messages[last_user].get('content', '')) and not any(m.get('role') == 'tool' for m in messages[last_user+1:])
        if should_ask and any(t.get('function', {}).get('name') == 'question' for t in request.get('tools', [])):
            args = {'questions': [{'header': 'View', 'question': 'Which view should remain active?', 'options': [{'label': 'Conversation', 'description': 'Keep the chat visible'}, {'label': 'Terminal', 'description': 'Reveal the existing terminal'}]}]}
            event = {'id': 'chatcmpl-fixture', 'object': 'chat.completion.chunk', 'created': 1, 'model': 'fixture', 'choices': [{'index': 0, 'delta': {'tool_calls': [{'index': 0, 'id': 'call_fixture_question', 'type': 'function', 'function': {'name': 'question', 'arguments': json.dumps(args)}}]}, 'finish_reason': 'tool_calls'}]}
            self.wfile.write(('data: ' + json.dumps(event) + '\n\ndata: [DONE]\n\n').encode())
            return
        for part in ['Same running ', 'conversation.\n\n', '**Verified** with a local fixture.\n\n', '```rust\nlet preserved = true;\n```']:
            event = {'id': 'chatcmpl-fixture', 'object': 'chat.completion.chunk', 'created': 1,
                     'model': 'fixture', 'choices': [{'index': 0, 'delta': {'content': part}, 'finish_reason': None}]}
            self.wfile.write(('data: ' + json.dumps(event) + '\n\n').encode())
            self.wfile.flush()
            time.sleep(0.2)
        event = {'id': 'chatcmpl-fixture', 'object': 'chat.completion.chunk', 'created': 1,
                 'model': 'fixture', 'choices': [{'index': 0, 'delta': {}, 'finish_reason': 'stop'}],
                 'usage': {'prompt_tokens': 12, 'completion_tokens': 10, 'total_tokens': 22}}
        self.wfile.write(('data: ' + json.dumps(event) + '\n\ndata: [DONE]\n\n').encode())


    def sse(self, events):
        self.send_response(200)
        self.send_header('Content-Type', 'text/event-stream')
        self.send_header('Connection', 'close')
        self.end_headers()
        for event in events:
            self.wfile.write(('event: ' + event['type'] + '\ndata: ' + json.dumps(event) + '\n\n').encode())
            self.wfile.flush()
            time.sleep(0.03)

    def respond_codex(self, request):
        text = 'Same terminal. Verified with a local fixture.'
        part = {'type': 'output_text', 'text': text, 'annotations': []}
        item = {'type': 'message', 'id': 'msg_fixture_' + str(time.time_ns()), 'role': 'assistant', 'status': 'completed', 'content': [part]}
        response = {'id': 'resp_fixture_' + str(time.time_ns()), 'object': 'response', 'created_at': int(time.time()), 'model': request.get('model', 'fixture'), 'status': 'completed', 'output': [item], 'usage': {'input_tokens': 12, 'output_tokens': 10, 'total_tokens': 22}}
        self.sse([
            {'type': 'response.created', 'response': {**response, 'status': 'in_progress', 'output': []}},
            {'type': 'response.output_item.added', 'output_index': 0, 'item': {**item, 'status': 'in_progress', 'content': []}},
            {'type': 'response.content_part.added', 'output_index': 0, 'content_index': 0, 'item_id': item['id'], 'part': {**part, 'text': ''}},
            {'type': 'response.output_text.delta', 'output_index': 0, 'content_index': 0, 'item_id': item['id'], 'delta': text},
            {'type': 'response.output_text.done', 'output_index': 0, 'content_index': 0, 'item_id': item['id'], 'text': text},
            {'type': 'response.content_part.done', 'output_index': 0, 'content_index': 0, 'item_id': item['id'], 'part': part},
            {'type': 'response.output_item.done', 'output_index': 0, 'item': item},
            {'type': 'response.completed', 'response': response},
        ])

    def anthropic(self, request):
        text = 'Same terminal. Verified with a local fixture.'
        message = {'id': 'msg_fixture_' + str(time.time_ns()), 'type': 'message', 'role': 'assistant', 'model': request.get('model', 'fixture'), 'content': [{'type': 'text', 'text': text}], 'stop_reason': 'end_turn', 'stop_sequence': None, 'usage': {'input_tokens': 12, 'output_tokens': 10}}
        if not request.get('stream') or 'count_tokens' in self.path:
            body=json.dumps({'input_tokens': 12} if 'count_tokens' in self.path else message).encode()
            self.send_response(200); self.send_header('Content-Type', 'application/json'); self.send_header('Content-Length', str(len(body))); self.end_headers(); self.wfile.write(body)
            return
        self.sse([
            {'type': 'message_start', 'message': {**message, 'content': [], 'stop_reason': None}},
            {'type': 'content_block_start', 'index': 0, 'content_block': {'type': 'text', 'text': ''}},
            {'type': 'content_block_delta', 'index': 0, 'delta': {'type': 'text_delta', 'text': text}},
            {'type': 'content_block_stop', 'index': 0},
            {'type': 'message_delta', 'delta': {'stop_reason': 'end_turn', 'stop_sequence': None}, 'usage': {'output_tokens': 10}},
            {'type': 'message_stop'},
        ])


server = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
print(server.server_port, flush=True)
server.serve_forever()
