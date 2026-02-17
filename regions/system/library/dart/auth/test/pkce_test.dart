import 'dart:math';

import 'package:k1s0_auth/auth.dart';
import 'package:test/test.dart';

void main() {
  group('PKCE', () {
    group('generateCodeVerifier', () {
      test('should generate a string of length 43', () {
        final verifier = generateCodeVerifier();
        expect(verifier.length, equals(43));
      });

      test('should only contain unreserved characters', () {
        final verifier = generateCodeVerifier();
        // RFC 7636: unreserved characters [A-Z] / [a-z] / [0-9] / "-" / "." / "_" / "~"
        expect(
          verifier,
          matches(RegExp(r'^[A-Za-z0-9\-._~]+$')),
        );
      });

      test('should generate unique values on each call', () {
        final v1 = generateCodeVerifier();
        final v2 = generateCodeVerifier();
        expect(v1, isNot(equals(v2)));
      });

      test('should use the injected random', () {
        // Fixed seed for deterministic output
        final rng = Random(42);
        final v1 = generateCodeVerifier(random: Random(42));
        final v2 = generateCodeVerifier(random: Random(42));
        expect(v1, equals(v2));

        // Different seed
        final v3 = generateCodeVerifier(random: rng);
        // After consuming from rng, a new Random(42) should differ
        final v4 = generateCodeVerifier(random: rng);
        expect(v3, isNot(equals(v4)));
      });
    });

    group('generateCodeChallenge', () {
      test('should generate a base64url-encoded SHA-256 hash', () {
        final verifier = 'dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk';
        final challenge = generateCodeChallenge(verifier);
        // base64url: [A-Za-z0-9_-] only, no padding
        expect(challenge, matches(RegExp(r'^[A-Za-z0-9_-]+$')));
      });

      test('should produce consistent output for the same input', () {
        const verifier = 'test-verifier-value';
        final c1 = generateCodeChallenge(verifier);
        final c2 = generateCodeChallenge(verifier);
        expect(c1, equals(c2));
      });

      test('should produce different output for different inputs', () {
        final c1 = generateCodeChallenge('verifier-1');
        final c2 = generateCodeChallenge('verifier-2');
        expect(c1, isNot(equals(c2)));
      });

      test('should not contain padding characters', () {
        for (var i = 0; i < 10; i++) {
          final verifier = generateCodeVerifier();
          final challenge = generateCodeChallenge(verifier);
          expect(challenge, isNot(contains('=')));
        }
      });

      test('should not contain + or / characters', () {
        for (var i = 0; i < 10; i++) {
          final verifier = generateCodeVerifier();
          final challenge = generateCodeChallenge(verifier);
          expect(challenge, isNot(contains('+')));
          expect(challenge, isNot(contains('/')));
        }
      });
    });

    group('base64UrlEncode', () {
      test('should encode an empty list', () {
        expect(base64UrlEncode([]), equals(''));
      });

      test('should encode without padding', () {
        final encoded = base64UrlEncode([1]);
        expect(encoded, isNot(contains('=')));
      });

      test('should use URL-safe characters', () {
        // Values known to produce + and / in standard base64
        final encoded = base64UrlEncode([251, 255, 254]);
        expect(encoded, isNot(contains('+')));
        expect(encoded, isNot(contains('/')));
      });
    });
  });
}
