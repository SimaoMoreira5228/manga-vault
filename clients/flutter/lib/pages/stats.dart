import 'package:flutter/material.dart';

import '../service/vault_service.dart';

class StatsPage extends StatefulWidget {
  const StatsPage({super.key, required this.vault});

  final VaultService vault;

  @override
  State<StatsPage> createState() => _StatsPageState();
}

class _StatsPageState extends State<StatsPage> {
  ReadingStatsSummary? stats;
  String? error;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    setState(() => error = null);
    try {
      final loaded = await widget.vault.readingStats();
      if (!mounted) return;
      setState(() => stats = loaded);
    } catch (e) {
      if (!mounted) return;
      setState(() => error = e.toString());
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final summary = stats;
    return Scaffold(
      appBar: AppBar(title: const Text('Statistics')),
      body: error != null
          ? Center(
              child: Padding(
                padding: const EdgeInsets.all(24),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Text(
                      'Could not load statistics',
                      style: theme.textTheme.titleMedium,
                    ),
                    const SizedBox(height: 8),
                    Text(
                      error!,
                      textAlign: TextAlign.center,
                      style: theme.textTheme.bodySmall,
                    ),
                    const SizedBox(height: 16),
                    FilledButton(onPressed: _load, child: const Text('Retry')),
                  ],
                ),
              ),
            )
          : summary == null
          ? const Center(child: CircularProgressIndicator())
          : ListView(
              padding: const EdgeInsets.all(16),
              children: [
                Row(
                  children: [
                    Expanded(
                      child: _StatTile(
                        label: 'Chapters read',
                        value: _grouped(summary.totalRead),
                      ),
                    ),
                    const SizedBox(width: 12),
                    Expanded(
                      child: _StatTile(
                        label: 'Reading streak',
                        value: '${summary.streak} days',
                      ),
                    ),
                    const SizedBox(width: 12),
                    Expanded(
                      child: _StatTile(
                        label: 'Works started',
                        value: '${summary.worksStarted}',
                      ),
                    ),
                  ],
                ),
                if (summary.daily.isNotEmpty) ...[
                  const SizedBox(height: 28),
                  Text('Last 30 days', style: theme.textTheme.titleMedium),
                  const SizedBox(height: 12),
                  DailyReadChart(daily: summary.daily),
                ],
              ],
            ),
    );
  }
}

class _StatTile extends StatelessWidget {
  const _StatTile({required this.label, required this.value});

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: theme.colorScheme.surfaceContainerLow,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: theme.colorScheme.outlineVariant),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            label,
            style: theme.textTheme.bodySmall?.copyWith(
              color: theme.colorScheme.onSurfaceVariant,
            ),
          ),
          const SizedBox(height: 8),
          Text(
            value,
            style: theme.textTheme.headlineSmall?.copyWith(
              fontWeight: FontWeight.bold,
            ),
          ),
        ],
      ),
    );
  }
}

class DailyReadChart extends StatelessWidget {
  const DailyReadChart({super.key, required this.daily});

  final Map<String, int> daily;

  @override
  Widget build(BuildContext context) {
    const bars = 120.0;
    final theme = Theme.of(context);
    final days = daily.entries.toList()..sort((a, b) => a.key.compareTo(b.key));
    final peak = days.fold<int>(
      1,
      (best, day) => day.value > best ? day.value : best,
    );
    return SizedBox(
      height: bars + 24,
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.end,
        children: [
          for (var index = 0; index < days.length; index++)
            Expanded(
              child: Tooltip(
                message: '${days[index].key}: ${days[index].value}',
                child: Column(
                  mainAxisAlignment: MainAxisAlignment.end,
                  children: [
                    Container(
                      height: bars * days[index].value / peak,
                      decoration: BoxDecoration(
                        color: theme.colorScheme.primary,
                        borderRadius: const BorderRadius.vertical(
                          top: Radius.circular(2),
                        ),
                      ),
                    ),
                    Text(
                      index % 5 == 0 || index == days.length - 1
                          ? _dayLabel(days[index].key)
                          : '',
                      style: theme.textTheme.bodySmall,
                    ),
                  ],
                ),
              ),
            ),
        ],
      ),
    );
  }
}

final _groups = RegExp(r'(\d)(?=(\d{3})+$)');

String _grouped(int value) =>
    value.toString().replaceAllMapped(_groups, (match) => '${match[1]},');

String _dayLabel(String day) => day.length > 5 ? day.substring(5) : day;
