import 'dart:async';

class SyncScheduler {
  Timer? _periodic;
  Timer? _debounce;
  Future<void> Function()? _task;
  bool _running = false;

  static final SyncScheduler instance = SyncScheduler._();

  SyncScheduler._();

  void start(
    Future<void> Function() task, {
    Duration interval = const Duration(minutes: 5),
  }) {
    stop();
    _task = task;
    _periodic = Timer.periodic(interval, (_) => run());
    run();
  }

  void nudge({Duration delay = const Duration(seconds: 8)}) {
    if (_task == null) return;
    _debounce?.cancel();
    _debounce = Timer(delay, run);
  }

  Future<void> run() async {
    final task = _task;
    if (task == null || _running) return;
    _running = true;
    try {
      await task();
    } catch (_) {
      // offline or provider hiccup: next tick retries
    } finally {
      _running = false;
    }
  }

  void stop() {
    _periodic?.cancel();
    _debounce?.cancel();
    _periodic = null;
    _debounce = null;
    _task = null;
  }
}
