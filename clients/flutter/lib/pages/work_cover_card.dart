import 'package:flutter/material.dart';

import '../service/vault_service.dart';

class WorkCoverCard extends StatelessWidget {
  const WorkCoverCard({
    super.key,
    required this.vault,
    required this.title,
    required this.coverUrl,
    required this.remoteUrl,
    this.kind,
    this.sourceId,
    this.badge,
    this.progress,
    this.onTap,
    this.onLongPress,
  });

  final VaultService vault;
  final String title;
  final String? coverUrl;
  final String remoteUrl;
  final String? kind;
  final String? sourceId;
  final String? badge;
  final ChapterProgressStats? progress;
  final VoidCallback? onTap;
  final VoidCallback? onLongPress;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final unread = _unread;
    return Semantics(
      button: onTap != null,
      label: [title, ?kind, if (unread != null) '$unread unread'].join(', '),
      excludeSemantics: true,
      child: InkWell(
        onTap: onTap,
        onLongPress: onLongPress,
        borderRadius: BorderRadius.circular(10),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            Expanded(
              child: Stack(
                fit: StackFit.expand,
                children: [
                  ClipRRect(
                    borderRadius: BorderRadius.circular(10),
                    child: _Cover(
                      vault: vault,
                      coverUrl: coverUrl,
                      remoteUrl: remoteUrl,
                    ),
                  ),
                  if (kind != null)
                    Positioned(
                      top: 6,
                      left: 6,
                      child: _Chip(
                        text: kind!,
                        background: kind == 'novel'
                            ? scheme.secondary.withValues(alpha: 0.22)
                            : scheme.primary.withValues(alpha: 0.22),
                        foreground: kind == 'novel'
                            ? scheme.secondary
                            : scheme.primary,
                      ),
                    ),
                  if (badge != null)
                    Positioned(
                      top: 6,
                      right: 6,
                      child: _Chip(
                        text: badge!,
                        background: Colors.black.withValues(alpha: 0.7),
                        foreground: scheme.onSurface,
                      ),
                    ),
                  if (unread != null)
                    Positioned(
                      right: 6,
                      bottom: 6,
                      child: _Chip(
                        text: '$unread',
                        background: scheme.primary,
                        foreground: scheme.onPrimary,
                      ),
                    ),
                ],
              ),
            ),
            const SizedBox(height: 6),
            Text(
              title,
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
              style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                fontWeight: FontWeight.w500,
                height: 1.25,
              ),
            ),
            if (progress != null && progress!.total > 0) ...[
              const SizedBox(height: 5),
              _Progress(
                read: progress!.read,
                total: progress!.total,
              ),
            ],
            if (sourceId != null)
              Padding(
                padding: const EdgeInsets.only(top: 3),
                child: Text(
                  sourceId!,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: Theme.of(context).textTheme.labelSmall?.copyWith(
                    color: scheme.outline,
                    letterSpacing: 0.5,
                  ),
                ),
              ),
          ],
        ),
      ),
    );
  }

  int? get _unread {
    final stats = progress;
    if (stats == null || stats.total == 0) return null;
    final unread = stats.total - stats.read;
    return unread > 0 ? unread : null;
  }
}

class _Cover extends StatelessWidget {
  const _Cover({
    required this.vault,
    required this.coverUrl,
    required this.remoteUrl,
  });

  final VaultService vault;
  final String? coverUrl;
  final String remoteUrl;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    if (coverUrl == null || coverUrl!.isEmpty) {
      return _Fallback(scheme: scheme);
    }
    return Image.network(
      vault.imageUrl(coverUrl!, referer: remoteUrl),
      headers: vault.imageHeaders,
      fit: BoxFit.cover,
      gaplessPlayback: true,
      errorBuilder: (_, _, _) => _Fallback(scheme: scheme),
      loadingBuilder: (context, child, progress) => progress == null
          ? child
          : ColoredBox(
              color: scheme.surfaceContainerHigh,
              child: const SizedBox.expand(),
            ),
    );
  }
}

class _Fallback extends StatelessWidget {
  const _Fallback({required this.scheme});

  final ColorScheme scheme;

  @override
  Widget build(BuildContext context) => DecoratedBox(
    decoration: BoxDecoration(
      color: scheme.surfaceContainerHigh,
      border: Border.all(color: scheme.outlineVariant.withValues(alpha: 0.4)),
      borderRadius: BorderRadius.circular(10),
    ),
    child: Center(
      child: Icon(
        Icons.auto_stories_outlined,
        size: 32,
        color: scheme.onSurfaceVariant.withValues(alpha: 0.5),
      ),
    ),
  );
}

class _Chip extends StatelessWidget {
  const _Chip({
    required this.text,
    required this.background,
    required this.foreground,
  });

  final String text;
  final Color background;
  final Color foreground;

  @override
  Widget build(BuildContext context) => DecoratedBox(
    decoration: BoxDecoration(
      color: background,
      borderRadius: BorderRadius.circular(4),
    ),
    child: Padding(
      padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 2),
      child: Text(
        text,
        style: Theme.of(context).textTheme.labelSmall?.copyWith(
          color: foreground,
          letterSpacing: 0.4,
        ),
      ),
    ),
  );
}

class _Progress extends StatelessWidget {
  const _Progress({required this.read, required this.total});

  final int read;
  final int total;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final value = (read / total).clamp(0.0, 1.0);
    return ClipRRect(
      borderRadius: BorderRadius.circular(2),
      child: LinearProgressIndicator(
        value: value,
        minHeight: 3,
        backgroundColor: scheme.surfaceContainerHighest,
        valueColor: AlwaysStoppedAnimation(scheme.primary),
      ),
    );
  }
}

class WorkCoverSliverGrid extends StatelessWidget {
  const WorkCoverSliverGrid({
    super.key,
    required this.itemCount,
    required this.itemBuilder,
    this.maxCrossAxisExtent = 148,
  });

  final int itemCount;
  final Widget Function(BuildContext context, int index) itemBuilder;
  final double maxCrossAxisExtent;

  @override
  Widget build(BuildContext context) {
    final width = MediaQuery.sizeOf(context).width;
    final extent = width >= 1000
        ? (maxCrossAxisExtent * 1.35).clamp(maxCrossAxisExtent, 220.0)
        : width >= 700
        ? (maxCrossAxisExtent * 1.15).clamp(maxCrossAxisExtent, 200.0)
        : maxCrossAxisExtent;
    return SliverGrid(
      gridDelegate: SliverGridDelegateWithMaxCrossAxisExtent(
        maxCrossAxisExtent: extent,
        mainAxisExtent: extent * 1.78,
        crossAxisSpacing: 12,
        mainAxisSpacing: 16,
      ),
      delegate: SliverChildBuilderDelegate(itemBuilder, childCount: itemCount),
    );
  }
}
