/// k1s0 Real-time Communication Library
///
/// Provides WebSocket and Server-Sent Events (SSE) clients with
/// automatic reconnection, heartbeat, offline queue, and network monitoring.
library k1s0_realtime;

// Types
export 'src/types/connection_info.dart';
export 'src/types/connection_status.dart';
export 'src/types/heartbeat_config.dart';
export 'src/types/offline_queue_config.dart';
export 'src/types/realtime_config.dart';
export 'src/types/reconnect_config.dart';
export 'src/types/sse_event.dart';

// Utils
export 'src/utils/backoff.dart';
export 'src/utils/network_monitor.dart';

// WebSocket
export 'src/websocket/k1s0_websocket.dart';

// SSE
export 'src/sse/k1s0_sse.dart';

// Provider / Manager
export 'src/provider/realtime_manager.dart';
export 'src/provider/offline_queue.dart';
