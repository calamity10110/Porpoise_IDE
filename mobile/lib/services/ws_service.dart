import 'dart:async';
import 'dart:convert';
import 'package:web_socket_channel/web_socket_channel.dart';

class WsService {
  WebSocketChannel? _channel;
  String? _serverUrl;
  String? _authToken;

  bool get isConnected => _channel != null;

  Future<void> connect(String host, int port, {String? token}) async {
    _serverUrl = 'ws://$host:$port';
    _authToken = token;
    final uri = Uri.parse('$_serverUrl');
    _channel = WebSocketChannel.connect(uri);
    await _channel!.ready;
  }

  Future<Map<String, dynamic>> call(String method, Map<String, dynamic> params) async {
    if (_channel == null) throw Exception('Not connected');

    final request = jsonEncode({
      'method': method,
      'params': params,
      'id': DateTime.now().millisecondsSinceEpoch.toString(),
    });

    _channel!.sink.add(request);

    final response = await _channel!.stream.first;
    return jsonDecode(response as String) as Map<String, dynamic>;
  }

  Stream<String> get eventStream {
    if (_channel == null) throw Exception('Not connected');
    return _channel!.stream.cast<String>();
  }

  void disconnect() {
    _channel?.sink.close();
    _channel = null;
  }
}
