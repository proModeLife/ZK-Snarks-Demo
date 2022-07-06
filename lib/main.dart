import 'dart:ffi';
import 'dart:io';
import 'package:flutter/material.dart';
import 'bridge_generated.dart';

const base = "native";
final path = Platform.isWindows ? "$base.dll" : "lib$base.so";
late final dylib = Platform.isIOS
    ? DynamicLibrary.process()
    : Platform.isMacOS
        ? DynamicLibrary.executable()
        : DynamicLibrary.open(path);
late final api = NativeImpl(dylib);

void main() {
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({Key? key}) : super(key: key);

  // This widget is the root of your application.
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Flutter Demo',
      theme: ThemeData(
        primarySwatch: Colors.blue,
      ),
      home: const MyHomePage(title: 'Flutter Demo Home Page'),
    );
  }
}

class MyHomePage extends StatefulWidget {
  const MyHomePage({Key? key, required this.title}) : super(key: key);

  final String title;

  @override
  State<MyHomePage> createState() => _MyHomePageState();
}

class _MyHomePageState extends State<MyHomePage> {
  late Future<int> counter;
  late bool temp;

  @override
  void initState() {
    super.initState();
    counter = api.getCounter();
  }

  void _incrementCounter() async {
    print('pressed');
    setState(() async {
      temp = (await api.testPedersenProof()) as bool;
      if (temp == true) {
        counter= api.increment();
        print('Proof checked');
      } else {
        print('Prove failed');
        counter = api.increment();
      }

    });
  }

  void _decrementCounter() {
    print('pressed');
    setState(() {
      counter = api.decrement();
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(widget.title),
      ),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: <Widget>[
            Container(
              child: IconButton(
                icon: Icon(Icons.volume_down),
                tooltip: "Decrement",
                onPressed: _decrementCounter,
              ),
            ),
            const Text(
              'You have pushed the button this many times:',
            ),
            FutureBuilder<List<dynamic>>(
                future: Future.wait([counter]),
                builder: (context, snap) {
                  final data = snap.data;
                  if (data == null) {
                    return const Text("Loading");
                  }
                  return Text(
                    '${data[0]}',
                    style: Theme.of(context).textTheme.headline4,
                  );
                }),
          ],
        ),
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: _incrementCounter,
        tooltip: 'Increment',
        child: const Icon(Icons.add),
      ),
    );
  }
}
