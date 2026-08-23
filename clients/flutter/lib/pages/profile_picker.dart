import 'package:flutter/material.dart';

import '../src/rust/api/local.dart' as local;
import '../service/local_service.dart';
import '../service/vault_service.dart';

class ProfilePickerPage extends StatefulWidget {
	const ProfilePickerPage({super.key, required this.vault, required this.profiles, required this.onSelected});

	final local.LocalVault vault;
	final List<local.ProfileSummary> profiles;
	final ValueChanged<VaultService> onSelected;

	@override
	State<ProfilePickerPage> createState() => _ProfilePickerPageState();
}

class _ProfilePickerPageState extends State<ProfilePickerPage> {
	late List<local.ProfileSummary> profiles;

	@override
	void initState() {
		super.initState();
		profiles = widget.profiles;
	}

	Future<void> _enter(local.ProfileSummary profile) async {
		if (profile.hasPin) {
			final pin = await _promptPin(profile.name);
			if (pin == null) return;
			try {
				await widget.vault.selectProfile(id: profile.id, pin: pin);
			} catch (_) {
				if (!mounted) return;
				ScaffoldMessenger.of(context).showSnackBar(const SnackBar(content: Text('Wrong PIN')));
				return;
			}
		} else {
			await widget.vault.selectProfile(id: profile.id);
		}
		if (!mounted) return;
		widget.onSelected(LocalService(widget.vault));
	}

	Future<String?> _promptPin(String name) async {
		final controller = TextEditingController();
		return showDialog<String>(
			context: context,
			builder: (context) => AlertDialog(
				title: Text('PIN for $name'),
				content: TextField(
					controller: controller,
					autofocus: true,
					obscureText: true,
					keyboardType: TextInputType.number,
					onSubmitted: (value) => Navigator.of(context).pop(value),
				),
				actions: [
					TextButton(onPressed: () => Navigator.of(context).pop(), child: const Text('Cancel')),
					FilledButton(onPressed: () => Navigator.of(context).pop(controller.text), child: const Text('Unlock')),
				],
			),
		);
	}

	Future<void> _createProfile() async {
		final name = TextEditingController();
		final pin = TextEditingController();
		await showDialog<void>(
			context: context,
			builder: (context) => AlertDialog(
				title: const Text('New profile'),
				content: Column(
					mainAxisSize: MainAxisSize.min,
					children: [
						TextField(controller: name, autofocus: true, decoration: const InputDecoration(hintText: 'Name')),
						const SizedBox(height: 12),
						TextField(
							controller: pin,
							obscureText: true,
							keyboardType: TextInputType.number,
							decoration: const InputDecoration(hintText: 'PIN (optional)'),
						),
					],
				),
				actions: [
					TextButton(onPressed: () => Navigator.of(context).pop(), child: const Text('Cancel')),
					FilledButton(
						onPressed: () async {
							final trimmed = name.text.trim();
							if (trimmed.isEmpty || !context.mounted) return;
							try {
								final created = await widget.vault.createProfile(name: trimmed, pin: pin.text.isEmpty ? null : pin.text);
								if (!context.mounted) return;
								Navigator.of(context).pop();
								setState(() => profiles = [...profiles, created]);
							} catch (e) {
								if (!context.mounted) return;
								ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('$e')));
							}
						},
						child: const Text('Create'),
					),
				],
			),
		);
	}

	@override
	Widget build(BuildContext context) {
		return Scaffold(
			body: Center(
				child: ConstrainedBox(
					constraints: const BoxConstraints(maxWidth: 480),
					child: Column(
						mainAxisAlignment: MainAxisAlignment.center,
						children: [
							Text('Who is reading?', style: Theme.of(context).textTheme.displaySmall),
							const SizedBox(height: 32),
							Wrap(
								spacing: 16,
								runSpacing: 16,
								alignment: WrapAlignment.center,
								children: [
									for (final profile in profiles)
										ActionChip(
											label: Padding(
												padding: const EdgeInsets.all(8),
												child: Column(
													mainAxisSize: MainAxisSize.min,
													children: [
														Icon(
															profile.hasPin ? Icons.lock_person : Icons.person,
															size: 40,
															color: Theme.of(context).colorScheme.primary,
														),
														const SizedBox(height: 8),
														Text(profile.name),
													],
												),
											),
											onPressed: () => _enter(profile),
										),
									ActionChip(
										label: const Padding(
											padding: EdgeInsets.all(8),
											child: Icon(Icons.add, size: 40),
										),
										onPressed: _createProfile,
									),
								],
							),
						],
					),
				),
			),
		);
	}
}
