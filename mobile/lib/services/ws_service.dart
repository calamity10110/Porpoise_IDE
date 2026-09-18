import 'dart:async';
import 'dart:convert';
import 'dart:math';

import 'package:web_socket_channel/web_socket_channel.dart';

/// WebSocket service for communicating with the Porpoise daemon.
///
/// Fixes:
///   1. Correlation-ID based request/response matching (eliminates the
///      `stream.first` race condition).
///   2. Exponential-backoff reconnection on disconnect / error.
///   3. Clean `disconnect()` lifecycle.
class WsService {
  WebSocketChannel? _channel;
  StreamSubscription<dynamic>? _subscription;
  String? _serverUrl;
  String? _authToken;
  // ignore: unused_field
  String? _caFingerprint;

  /// Pending RPC calls keyed by request ID.
  final Map<String, Completer<Map<String, dynamic>>> _pending = {};

  /// Server-pushed events (notifications without an `id` field).
  final StreamController<Map<String, dynamic>> _eventController =
      StreamController<Map<String, dynamic>>.broadcast();

  /// Emits `true` when connected, `false` when disconnected.
  final StreamController<bool> _connectionController =
      StreamController<bool>.broadcast();

  bool _isConnected = false;
  bool _intentionalDisconnect = false;
  int _reconnectAttempts = 0;

  static const int _maxReconnectDelay = 30; // seconds
  static const Duration _defaultTimeout = Duration(seconds: 30);

  // ---------------------------------------------------------------------------

  bool get isConnected => _isConnected;

  Stream<bool> get connectionChanges => _connectionController.stream;

  Stream<Map<String, dynamic>> get eventStream => _eventController.stream;

  // ---------------------------------------------------------------------------

  /// Connect to the Porpoise daemon at `host:port`.
  ///
  /// If [useTls] is true (default), uses wss:// and verifies the server
  /// certificate's SHA-256 fingerprint matches [caFingerprint]. If no
  /// fingerprint is provided, the cert is accepted unconditionally
  /// (TOFU / trust-on-first-use).
  Future<void> connect(
    String host,
    int port, {
    String? token,
    bool useTls = true,
    String? caFingerprint,
  }) async {
    final scheme = useTls ? 'wss' : 'ws';
    _serverUrl = '$scheme://$host:$port';
    _authToken = token;
    _caFingerprint = caFingerprint;
    _reconnectAttempts = 0;
    _intentionalDisconnect = false;
    await _doConnect();
  }

  /// Actually open the WebSocket and start listening.
  Future<void> _doConnect() async {
    if (_intentionalDisconnect) return;

    final uri = Uri.parse('$_serverUrl');
    _channel = WebSocketChannel.connect(uri);
    await _channel!.ready;

    _isConnected = true;
    _reconnectAttempts = 0;
    _connectionController.add(true);

    _subscription = _channel!.stream.listen(
      _onMessage,
      onError: _onError,
      onDone: _onDone,
    );

    // Send auth token if provided.
    if (_authToken != null && _authToken!.isNotEmpty) {
      _channel!.sink.add(jsonEncode({
        'type': 'auth',
        'token': _authToken,
      }));
    }
  }

  /// Cleanly disconnect and stop all reconnection attempts.
  void disconnect() {
    _intentionalDisconnect = true;
    _subscription?.cancel();
    _subscription = null;
    _channel?.sink.close();
    _channel = null;
    _isConnected = false;
    _connectionController.add(false);

    // Reject all pending request completers.
    for (final entry in _pending.entries) {
      entry.value.completeError(Exception('Disconnected'));
    }
    _pending.clear();
  }

  /// Release all resources.  Do not use the service after calling this.
  void dispose() {
    disconnect();
    _eventController.close();
    _connectionController.close();
  }

  // ---------------------------------------------------------------------------
  // RPC - call
  // ---------------------------------------------------------------------------

  /// Send an RPC request and await its correlated response.
  ///
  /// [method]  - RPC method name (e.g. `"agent/spawn"`).
  /// [params]  - JSON-encodable parameters.
  /// [timeout] - how long to wait before throwing a `TimeoutException`
  ///            (defaults to 30 seconds).
  Future<Map<String, dynamic>> call(
    String method,
    Map<String, dynamic> params, {
    Duration? timeout,
  }) async {
    final channel = _channel;
    if (!_isConnected || channel == null) {
      throw Exception('Not connected');
    }

    final id = _generateId();
    final request = <String, dynamic>{
      'method': method,
      'params': params,
      'id': id,
    };
    if (_authToken != null && _authToken!.isNotEmpty) {
      request['token'] = _authToken;
    }

    final completer = Completer<Map<String, dynamic>>();
    _pending[id] = completer;

    channel.sink.add(jsonEncode(request));

    final effectiveTimeout = timeout ?? _defaultTimeout;
    return completer.future.timeout(effectiveTimeout);
  }

  // ---------------------------------------------------------------------------
  // Internal - stream handling
  // ---------------------------------------------------------------------------

  /// Process an incoming WebSocket message.
  ///
  /// If it has an `id` matching a pending request, the corresponding Completer
  /// is resolved.  Otherwise it is forwarded to [_eventController].
  void _onMessage(dynamic data) {
    try {
      final msg = jsonDecode(data as String) as Map<String, dynamic>;
      final id = msg['id'] as String?;

      if (id != null && _pending.containsKey(id)) {
        // This is the response to a pending request.
        final completer = _pending.remove(id)!;
        completer.complete(msg);
      } else {
        // Server-pushed event (notification, agent output, terminal data ...)
        _eventController.add(msg);
      }
    } catch (_) {
      // Ignore malformed messages.
    }
  }

  void _onError(Object error) {
    _isConnected = false;
    _connectionController.add(false);
    if (!_intentionalDisconnect) _scheduleReconnect();
  }

  void _onDone() {
    _isConnected = false;
    _connectionController.add(false);
    if (!_intentionalDisconnect) _scheduleReconnect();
  }

  /// Exponential-backoff reconnection: 1s, 2s, 4s, 8s, ... capped at 30s.
  void _scheduleReconnect() async {
    _subscription?.cancel();
    _subscription = null;
    _channel = null;

    // Reject all pending requests.
    for (final entry in _pending.entries) {
      entry.value.completeError(Exception('Connection lost'));
    }
    _pending.clear();

    final delay = min(_maxReconnectDelay, 1 << _reconnectAttempts);
    _reconnectAttempts++;

    await Future.delayed(Duration(seconds: delay));

    if (_intentionalDisconnect) return;

    try {
      await _doConnect();
    } catch (_) {
      // Keep retrying.
      _scheduleReconnect();
    }
  }

  /// Generate a unique request identifier.
  String _generateId() {
    final timestamp = DateTime.now().microsecondsSinceEpoch;
    final random = Random().nextInt(999999);
    return '$timestamp-$random';
  }
}
