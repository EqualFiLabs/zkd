import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:zkprov_flutter_example/main.dart';

void main() {
  testWidgets('renders placeholder text', (WidgetTester tester) async {
    await tester.pumpWidget(const MyApp());

    expect(
      find.byWidgetPredicate(
        (Widget widget) =>
            widget is Text && widget.data == 'ZKProv Flutter FFI example placeholder',
      ),
      findsOneWidget,
    );
  });
}
