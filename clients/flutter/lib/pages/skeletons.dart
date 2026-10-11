import 'package:flutter/material.dart';

class SkeletonBar extends StatelessWidget {
  const SkeletonBar({
    super.key,
    required this.height,
    this.width = double.infinity,
    this.radius = 4,
  });

  final double height;
  final double width;
  final double radius;

  @override
  Widget build(BuildContext context) {
    return Container(
      width: width,
      height: height,
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHigh,
        borderRadius: BorderRadius.circular(radius),
      ),
    );
  }
}

class SkeletonTile extends StatelessWidget {
  const SkeletonTile({
    super.key,
    this.leading,
    this.trailing,
    this.subtitle = true,
  });

  final Widget? leading;
  final Widget? trailing;
  final bool subtitle;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
      child: Row(
        children: [
          if (leading != null) ...[leading!, const SizedBox(width: 16)],
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const FractionallySizedBox(
                  widthFactor: 0.55,
                  child: SkeletonBar(height: 22),
                ),
                if (subtitle) ...[
                  const SizedBox(height: 6),
                  const FractionallySizedBox(
                    widthFactor: 0.35,
                    child: SkeletonBar(height: 14),
                  ),
                ],
              ],
            ),
          ),
          if (trailing != null) ...[const SizedBox(width: 16), trailing!],
        ],
      ),
    );
  }
}

class SkeletonWorkCard extends StatelessWidget {
  const SkeletonWorkCard({super.key});

  @override
  Widget build(BuildContext context) {
    return DecoratedBox(
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHigh,
        borderRadius: BorderRadius.circular(8),
      ),
    );
  }
}

class SkeletonColumn extends StatelessWidget {
  const SkeletonColumn({
    super.key,
    required this.label,
    required this.count,
    required this.itemBuilder,
  });

  final String label;
  final int count;
  final IndexedWidgetBuilder itemBuilder;

  @override
  Widget build(BuildContext context) {
    return Semantics(
      container: true,
      label: label,
      child: ExcludeSemantics(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            for (var index = 0; index < count; index += 1)
              itemBuilder(context, index),
          ],
        ),
      ),
    );
  }
}

class SkeletonGrid extends StatelessWidget {
  const SkeletonGrid({super.key, required this.label, this.count = 8});

  final String label;
  final int count;

  @override
  Widget build(BuildContext context) {
    return Semantics(
      container: true,
      label: label,
      child: ExcludeSemantics(
        child: GridView.builder(
          padding: const EdgeInsets.all(16),
          gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
            maxCrossAxisExtent: 160,
            childAspectRatio: 2 / 3,
            crossAxisSpacing: 12,
            mainAxisSpacing: 12,
          ),
          itemCount: count,
          itemBuilder: (context, index) => const SkeletonWorkCard(),
        ),
      ),
    );
  }
}

class SkeletonList extends StatelessWidget {
  const SkeletonList({
    super.key,
    required this.label,
    required this.count,
    required this.itemBuilder,
  });

  final String label;
  final int count;
  final IndexedWidgetBuilder itemBuilder;

  @override
  Widget build(BuildContext context) {
    return Semantics(
      container: true,
      label: label,
      child: ExcludeSemantics(
        child: ListView.builder(
          padding: const EdgeInsets.symmetric(vertical: 8),
          itemCount: count,
          itemBuilder: itemBuilder,
        ),
      ),
    );
  }
}
