import 'package:flutter/material.dart';

import '../service/remote_service.dart';
import '../service/vault_service.dart';

class SourcesPage extends StatefulWidget {
  const SourcesPage({super.key, required this.vault});

  final VaultService vault;

  @override
  State<SourcesPage> createState() => _SourcesPageState();
}

class _SourcesPageState extends State<SourcesPage> {
  List<PluginRepo>? repos;
  List<CatalogItem>? catalog;
  String? error;
  bool operatorManaged = false;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    setState(() {
      repos = null;
      catalog = null;
      error = null;
    });
    try {
      final loadedRepos = await widget.vault.pluginRepos();
      final loadedCatalog = await widget.vault.pluginCatalog().catchError(
        (_) => <CatalogItem>[],
      );
      if (!mounted) return;
      setState(() {
        repos = loadedRepos;
        catalog = loadedCatalog;
        operatorManaged = false;
      });
    } on RemoteApiException catch (e) {
      if (!mounted) return;
      setState(() {
        repos = const [];
        catalog = const [];
        operatorManaged = e.statusCode == 403;
        error = e.statusCode == 403 ? null : e.toString();
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        repos = const [];
        catalog = const [];
        error = e.toString();
      });
    }
  }

  Future<void> _addRepo() async {
    final controller = TextEditingController();
    final url = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Add plugin repository'),
        content: TextField(
          controller: controller,
          autofocus: true,
          decoration: const InputDecoration(
            hintText: 'https://example.org/repo.json',
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () => Navigator.of(context).pop(controller.text.trim()),
            child: const Text('Add'),
          ),
        ],
      ),
    );
    if (url == null || url.isEmpty) return;
    await widget.vault.addPluginRepo(url: url);
    await _load();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Plugins'),
        actions: [
          if (!operatorManaged)
            IconButton(icon: const Icon(Icons.add), onPressed: _addRepo),
        ],
      ),
      body: error != null
          ? Center(
              child: Padding(
                padding: const EdgeInsets.all(24),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Text(
                      'Failed to load plugins',
                      style: Theme.of(context).textTheme.titleMedium,
                    ),
                    const SizedBox(height: 8),
                    Text(
                      error!,
                      textAlign: TextAlign.center,
                      style: Theme.of(context).textTheme.bodySmall,
                    ),
                    const SizedBox(height: 16),
                    FilledButton(
                      onPressed: _load,
                      child: const Text('Retry'),
                    ),
                  ],
                ),
              ),
            )
          : CustomScrollView(
              slivers: [
                if (operatorManaged)
                  const SliverToBoxAdapter(
                    child: Padding(
                      padding: EdgeInsets.fromLTRB(16, 16, 16, 8),
                      child: Text(
                        'Plugins are managed by this server\'s operator.',
                      ),
                    ),
                  ),
                const SliverToBoxAdapter(
                  child: Padding(
                    padding: EdgeInsets.fromLTRB(16, 16, 16, 8),
                    child: Text(
                      'REPOSITORIES',
                      style: TextStyle(
                        fontSize: 12,
                        fontWeight: FontWeight.w500,
                        letterSpacing: 0.6,
                      ),
                    ),
                  ),
                ),
                _buildRepos(),
                const SliverToBoxAdapter(
                  child: Padding(
                    padding: EdgeInsets.fromLTRB(16, 16, 16, 8),
                    child: Text(
                      'CATALOG',
                      style: TextStyle(
                        fontSize: 12,
                        fontWeight: FontWeight.w500,
                        letterSpacing: 0.6,
                      ),
                    ),
                  ),
                ),
                _buildCatalog(),
              ],
            ),
    );
  }

  SliverList _buildRepos() {
    final shown = repos;
    if (shown == null) {
      return SliverList(
        delegate: SliverChildListDelegate([
          Center(child: CircularProgressIndicator()),
        ]),
      );
    }
    if (shown.isEmpty) {
      return SliverList(
        delegate: SliverChildListDelegate([
          Padding(
            padding: EdgeInsets.symmetric(horizontal: 16),
            child: Text(
              'No repositories configured. Add one to browse its plugins.',
            ),
          ),
        ]),
      );
    }
    return SliverList(
      delegate: SliverChildBuilderDelegate(
        (context, index) => ListTile(
          title: Text(shown[index].name),
          subtitle: Text(shown[index].url, overflow: TextOverflow.ellipsis),
          trailing: IconButton(
            icon: const Icon(Icons.delete_outline),
            onPressed: () => _removeRepo(shown[index].id),
          ),
        ),
        childCount: shown.length,
      ),
    );
  }

  SliverList _buildCatalog() {
    final shown = catalog;
    if (shown == null)
      return SliverList(
        delegate: SliverChildListDelegate([
          Center(child: CircularProgressIndicator()),
        ]),
      );
    if (shown.isEmpty) {
      return SliverList(
        delegate: SliverChildListDelegate([
          Padding(
            padding: EdgeInsets.symmetric(horizontal: 16),
            child: Text('Nothing in the catalog yet.'),
          ),
        ]),
      );
    }
    return SliverList(
      delegate: SliverChildBuilderDelegate(
        (context, index) => CatalogTile(item: shown[index], onAction: _act),
        childCount: shown.length,
      ),
    );
  }

  Future<void> _removeRepo(String repoId) async {
    await widget.vault.removePluginRepo(repoId: repoId);
    await _load();
  }

  Future<void> _act(CatalogItem item) async {
    if (item.installedVersion != null && !item.updateAvailable) {
      await widget.vault.uninstallPlugin(pluginId: item.id);
    } else {
      await widget.vault.installPlugin(pluginId: item.id, repoId: item.repoId);
    }
    await _load();
  }
}

class CatalogTile extends StatelessWidget {
  const CatalogTile({super.key, required this.item, required this.onAction});

  final CatalogItem item;
  final Future<void> Function(CatalogItem) onAction;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final installed = item.installedVersion != null;
    final label = !installed
        ? 'Install'
        : item.updateAvailable
        ? 'Update'
        : 'Uninstall';
    return ListTile(
      title: Text(item.id),
      subtitle: Text(
        '${item.repoName} · v${item.availableVersion} · ${item.backend}',
      ),
      trailing: FilledButton.tonal(
        style: FilledButton.styleFrom(
          foregroundColor: installed && !item.updateAvailable
              ? scheme.error
              : scheme.primary,
        ),
        onPressed: () => onAction(item),
        child: Text(label),
      ),
    );
  }
}
