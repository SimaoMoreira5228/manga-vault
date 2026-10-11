import 'package:flutter/material.dart';

class PagerBar extends StatelessWidget {
  const PagerBar({
    super.key,
    required this.offset,
    required this.limit,
    required this.total,
    required this.onshift,
  });

  final int offset;
  final int limit;
  final int total;
  final ValueChanged<int> onshift;

  @override
  Widget build(BuildContext context) {
    if (total < 1) return const SizedBox.shrink();
    final theme = Theme.of(context);
    final last = offset + limit < total ? offset + limit : total;
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 8, 16, 16),
      child: Row(
        children: [
          OutlinedButton(
            onPressed: offset < limit ? null : () => onshift(offset - limit),
            child: const Text('Previous'),
          ),
          Expanded(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Text(
                  'Page ${offset ~/ limit + 1} of ${(total + limit - 1) ~/ limit}',
                  style: theme.textTheme.bodySmall,
                ),
                Text(
                  '${offset + 1}-$last of $total',
                  style: theme.textTheme.bodySmall?.copyWith(
                    color: theme.colorScheme.outline,
                  ),
                ),
              ],
            ),
          ),
          OutlinedButton(
            onPressed: offset + limit >= total
                ? null
                : () => onshift(offset + limit),
            child: const Text('Next'),
          ),
        ],
      ),
    );
  }
}
