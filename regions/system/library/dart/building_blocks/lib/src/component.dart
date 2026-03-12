enum ComponentStatus {
  uninitialized,
  ready,
  degraded,
  closed,
  error,
}

abstract class Component {
  String get name;
  String get componentType;
  Future<void> init();
  Future<void> close();
  Future<ComponentStatus> status();
  Map<String, String> metadata();
}
