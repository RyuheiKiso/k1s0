import 'dart:convert';
import 'dart:math';
import 'dart:typed_data';

import 'package:pointycastle/export.dart';
import 'package:pointycastle/asn1.dart';

/// Generates an RSA key pair (2048-bit) and returns PEM-encoded strings.
Map<String, String> generateRsaKeyPair() {
  final keyGen = RSAKeyGenerator();
  final secureRandom = FortunaRandom();

  final seedSource = Random.secure();
  final seeds = List<int>.generate(32, (_) => seedSource.nextInt(256));
  secureRandom.seed(KeyParameter(Uint8List.fromList(seeds)));

  keyGen.init(ParametersWithRandom(
    RSAKeyGeneratorParameters(BigInt.from(65537), 2048, 64),
    secureRandom,
  ));

  final pair = keyGen.generateKeyPair();
  final publicKey = pair.publicKey as RSAPublicKey;
  final privateKey = pair.privateKey as RSAPrivateKey;

  return {
    'publicKey': _encodePublicKeyToPem(publicKey),
    'privateKey': _encodePrivateKeyToPem(privateKey),
  };
}

/// Encrypts [plaintext] using RSA-OAEP with SHA-256.
Uint8List rsaEncrypt(String publicKeyPem, Uint8List plaintext) {
  final publicKey = _decodePublicKeyFromPem(publicKeyPem);
  final cipher = OAEPEncoding.withSHA256(RSAEngine());
  cipher.init(true, PublicKeyParameter<RSAPublicKey>(publicKey));
  return cipher.process(plaintext);
}

/// Decrypts [ciphertext] using RSA-OAEP with SHA-256.
Uint8List rsaDecrypt(String privateKeyPem, Uint8List ciphertext) {
  final privateKey = _decodePrivateKeyFromPem(privateKeyPem);
  final cipher = OAEPEncoding.withSHA256(RSAEngine());
  cipher.init(false, PrivateKeyParameter<RSAPrivateKey>(privateKey));
  return cipher.process(ciphertext);
}

/// Encodes an RSA public key to PKCS#8 / X.509 SubjectPublicKeyInfo PEM.
String _encodePublicKeyToPem(RSAPublicKey key) {
  final algorithmSeq = ASN1Sequence();
  algorithmSeq
      .add(ASN1ObjectIdentifier.fromIdentifierString('1.2.840.113549.1.1.1'));
  algorithmSeq.add(ASN1Null());

  final pubKeySeq = ASN1Sequence();
  pubKeySeq.add(ASN1Integer(key.modulus!));
  pubKeySeq.add(ASN1Integer(key.exponent!));

  final topSeq = ASN1Sequence();
  topSeq.add(algorithmSeq);
  topSeq.add(
      ASN1BitString(stringValues: pubKeySeq.encode()));

  final bytes = topSeq.encode();
  final b64 = base64Encode(bytes);
  final wrapped =
      RegExp(r'.{1,64}').allMatches(b64).map((m) => m.group(0)!).join('\n');
  return '-----BEGIN PUBLIC KEY-----\n$wrapped\n-----END PUBLIC KEY-----\n';
}

/// Encodes an RSA private key to PKCS#1 PEM.
String _encodePrivateKeyToPem(RSAPrivateKey key) {
  final seq = ASN1Sequence();
  seq.add(ASN1Integer(BigInt.zero));
  seq.add(ASN1Integer(key.modulus!));
  seq.add(ASN1Integer(key.publicExponent!));
  seq.add(ASN1Integer(key.privateExponent!));
  seq.add(ASN1Integer(key.p!));
  seq.add(ASN1Integer(key.q!));
  seq.add(ASN1Integer(key.privateExponent! % (key.p! - BigInt.one)));
  seq.add(ASN1Integer(key.privateExponent! % (key.q! - BigInt.one)));
  seq.add(ASN1Integer(key.q!.modInverse(key.p!)));

  final bytes = seq.encode();
  final b64 = base64Encode(bytes);
  final wrapped =
      RegExp(r'.{1,64}').allMatches(b64).map((m) => m.group(0)!).join('\n');
  return '-----BEGIN RSA PRIVATE KEY-----\n$wrapped\n-----END RSA PRIVATE KEY-----\n';
}

/// Decodes a PEM-encoded X.509 SubjectPublicKeyInfo to RSAPublicKey.
RSAPublicKey _decodePublicKeyFromPem(String pem) {
  final lines =
      pem.split('\n').where((l) => !l.startsWith('-----') && l.isNotEmpty).join();
  final bytes = base64Decode(lines);

  final asn1Seq = ASN1Parser(Uint8List.fromList(bytes)).nextObject() as ASN1Sequence;
  final bitString = asn1Seq.elements![1] as ASN1BitString;
  final pubKeyBytes = Uint8List.fromList(bitString.stringValues!);

  final pubKeySeq =
      ASN1Parser(pubKeyBytes).nextObject() as ASN1Sequence;
  final modulus = (pubKeySeq.elements![0] as ASN1Integer).integer!;
  final exponent = (pubKeySeq.elements![1] as ASN1Integer).integer!;

  return RSAPublicKey(modulus, exponent);
}

/// Decodes a PEM-encoded PKCS#1 RSA private key to RSAPrivateKey.
RSAPrivateKey _decodePrivateKeyFromPem(String pem) {
  final lines =
      pem.split('\n').where((l) => !l.startsWith('-----') && l.isNotEmpty).join();
  final bytes = base64Decode(lines);

  final seq = ASN1Parser(Uint8List.fromList(bytes)).nextObject() as ASN1Sequence;
  final modulus = (seq.elements![1] as ASN1Integer).integer!;
  final privateExponent = (seq.elements![3] as ASN1Integer).integer!;
  final p = (seq.elements![4] as ASN1Integer).integer!;
  final q = (seq.elements![5] as ASN1Integer).integer!;

  return RSAPrivateKey(modulus, privateExponent, p, q);
}
