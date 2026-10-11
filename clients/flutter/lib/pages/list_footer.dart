import 'package:flutter/material.dart';

class ListFooter extends StatelessWidget {
  const ListFooter({super.key, required this.busy, this.failure, this.onRetry});

  final bool busy;
  final String? failure;
  final VoidCallback? onRetry;

  @override
  Widget build(BuildContext context) {
    final problem = failure;
    if (problem != null) {
      return Semantics(
        liveRegion: true,
        child: Padding(
          padding: const EdgeInsets.fromLTRB(16, 12, 16, 16),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(
                'Could not load more: $problem',
                textAlign: TextAlign.center,
                style: Theme.of(context).textTheme.bodySmall,
              ),
              const SizedBox(height: 8),
              OutlinedButton(onPressed: onRetry, child: const Text('Retry')),
            ],
          ),
        ),
      );
    }
    if (!busy) return const SizedBox.shrink();
    return Semantics(
      liveRegion: true,
      label: 'Loading more',
      child: const Padding(
        padding: EdgeInsets.fromLTRB(16, 12, 16, 16),
        child: Center(
          child: SizedBox(
            width: 20,
            height: 20,
            child: CircularProgressIndicator(strokeWidth: 2),
          ),
        ),
      ),
    );
  }
}
