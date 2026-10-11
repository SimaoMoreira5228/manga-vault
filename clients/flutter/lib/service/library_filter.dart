import '../src/rust/api/local.dart';

String libraryQuerySuffix(LibraryFilter filter) {
  final text = filter.query?.trim();
  final parts = [
    if (filter.kind != null) 'kind=${filter.kind}',
    if (filter.categoryId != null)
      'category_id=${Uri.encodeQueryComponent(filter.categoryId!)}',
    if (text != null && text.isNotEmpty) 'q=${Uri.encodeQueryComponent(text)}',
  ];
  return parts.isEmpty ? '' : '&${parts.join('&')}';
}
