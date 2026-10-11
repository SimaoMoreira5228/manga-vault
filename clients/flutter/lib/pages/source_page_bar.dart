import 'package:flutter/material.dart';

class SourcePageBar extends StatelessWidget {
  const SourcePageBar({
    super.key,
    required this.page,
    required this.canAdvance,
    required this.onshift,
  });

  final int page;
  final bool canAdvance;
  final ValueChanged<int> onshift;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 8, 16, 16),
      child: Row(
        children: [
          OutlinedButton(
            onPressed: page < 2 ? null : () => onshift(page - 1),
            child: const Text('Previous'),
          ),
          Expanded(
            child: Text(
              'Page $page',
              textAlign: TextAlign.center,
              style: Theme.of(context).textTheme.bodySmall,
            ),
          ),
          OutlinedButton(
            onPressed: canAdvance ? () => onshift(page + 1) : null,
            child: const Text('Next'),
          ),
        ],
      ),
    );
  }
}