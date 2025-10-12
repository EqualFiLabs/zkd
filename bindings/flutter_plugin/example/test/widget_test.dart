import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:zkprov_flutter_example/main.dart';

void main() {
  testWidgets('renders initial dashboard state', (WidgetTester tester) async {
    await tester.pumpWidget(const MyApp());

    expect(find.text('ZKProv Demo'), findsOneWidget);
    expect(find.text('Backends JSON length: -'), findsOneWidget);
    expect(find.text('Last digest D: -'), findsOneWidget);
    expect(find.text('Verified: -'), findsOneWidget);
    expect(find.text('Ready'), findsOneWidget);
  });
}
