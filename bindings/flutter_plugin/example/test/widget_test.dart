import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:zkprov_flutter_example/main.dart';

void main() {
  testWidgets('renders initial dashboard state', (WidgetTester tester) async {
    await tester.pumpWidget(const MyApp());

    expect(find.text('ZKProv Demo'), findsOneWidget);
    expect(find.text('Backends available: -'), findsOneWidget);
    expect(find.text('First backend id: -'), findsOneWidget);
    expect(find.text('Digest D: -'), findsOneWidget);
    expect(find.text('Verified: -'), findsOneWidget);
    expect(find.text('Ready'), findsOneWidget);
  });
}
