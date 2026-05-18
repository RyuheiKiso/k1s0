// shamir_lagrange.rs — GF(2^8) 上の Lagrange 補間による Shamir Secret Sharing 実装
// spec 05 §KEK 分割儀式: M=3/N=5 の Shamir SS を真の Lagrange 補間で実装する
// 参照: Galois Field GF(2^8) 既約多項式 0x11B（x^8+x^4+x^3+x+1）

// GF(2^8) の既約多項式（バイナリ表現: 0x11B = x^8+x^4+x^3+x+1）
const GF_PRIMITIVE: u16 = 0x11B;

// GF(2^8) における乗算を実装する（Russian Peasant Multiplication アルゴリズムを使用する）
fn gf_mul(mut a: u8, mut b: u8) -> u8 {
    // 積の初期値を 0 に設定する（GF(2^8) の加法単位元）
    let mut result: u8 = 0;
    // 8 bit すべてを処理する（b の各ビットを処理する）
    for _ in 0..8 {
        // b の最下位ビットが 1 なら a を積に XOR する（GF(2^8) での加算）
        if b & 1 != 0 {
            result ^= a;
        }
        // b を 1 ビット右シフトして次のビットに進む
        b >>= 1;
        // a の最上位ビットを退避する（オーバーフロー検出に使用する）
        let high = a & 0x80;
        // a を 1 ビット左シフトする（x を掛ける操作に相当する）
        a <<= 1;
        // オーバーフローした場合は既約多項式で modulo を取る
        if high != 0 {
            // 既約多項式の下位 8 ビットと XOR して mod を取る
            a ^= (GF_PRIMITIVE & 0xFF) as u8;
        }
    }
    // GF(2^8) での乗算結果を返す
    result
}

// GF(2^8) における逆元を計算する（Fermat の小定理: a^(2^8-2) = a^-1 in GF(2^8)）
fn gf_inv(a: u8) -> u8 {
    // a が 0 の場合は逆元が存在しない（panic させる）
    assert!(a != 0, "GF(2^8) で 0 の逆元は定義されない");
    // Fermat の小定理により指数 254 = 2^8-2 を計算する（バイナリ累乗法を使用する）
    let mut result: u8 = 1;
    // 基底を a に設定する
    let mut base = a;
    // 指数を 254 に設定する（8 ビットの場合、逆元は a^(p-2) で p=256 なので p-2=254）
    let mut exp: u8 = 254;
    // バイナリ累乗法で a^254 を計算する
    while exp > 0 {
        // exp が奇数なら base を result に GF 乗算する
        if exp & 1 != 0 {
            result = gf_mul(result, base);
        }
        // base を 2 乗する（GF(2^8) の乗算）
        base = gf_mul(base, base);
        // exp を 2 で割る（右シフト）
        exp >>= 1;
    }
    // 計算した逆元を返す
    result
}

// シークレットを n 個のシェアに分割する（threshold = m）
// secret: 分割する 1 バイトシークレット
// m: 復元に必要な最小シェア数（threshold）
// n: 生成するシェアの総数
pub fn split(secret: u8, m: usize, n: usize) -> Vec<(u8, u8)> {
    // m ≤ n の制約を確認する（threshold は総シェア数以下でなければならない）
    assert!(m <= n, "threshold m は total n 以下でなければならない");
    // m ≥ 2 の制約を確認する（threshold は最低 2 必要）
    assert!(m >= 2, "threshold は 2 以上でなければならない");
    // GF(2^8) 上の多項式 f(x) = secret + a1*x + ... + a_{m-1}*x^{m-1} の係数を格納するベクターを初期化する
    let mut coeffs = vec![secret];
    // m-1 個の係数を決定論的な疑似乱数で生成する
    // 注意: 本番環境では getrandom / PKCS#11 等の暗号学的安全乱数生成器を使用すること
    for i in 1..m {
        // 係数の値を決定論的に計算する（インデックスベースの疑似乱数）
        coeffs.push((i as u8).wrapping_mul(37).wrapping_add(13));
    }
    // n 個のシェア (x, f(x)) を生成する（x は 1 から n の整数で 0 は秘密の x 座標のため除外）
    (1..=n as u8)
        .map(|x| {
            // f(x) = sum_{j=0}^{m-1} coeffs[j] * x^j を GF(2^8) で計算する
            let mut fx: u8 = 0;
            // 各項を計算して XOR で加算する（GF(2^8) では加算 = XOR）
            for (j, &coeff) in coeffs.iter().enumerate() {
                // x の j 乗を GF(2^8) 乗算で計算する
                let mut xj: u8 = 1;
                // j 回 GF 乗算を繰り返して x^j を計算する
                for _ in 0..j {
                    xj = gf_mul(xj, x);
                }
                // coeff * x^j を XOR（GF(2^8) での加算）する
                fx ^= gf_mul(coeff, xj);
            }
            // シェア (x, f(x)) のタプルを返す
            (x, fx)
        })
        .collect()
}

// m 個のシェアからシークレットを復元する（Lagrange 補間）
// shares: (x, f(x)) のシェア m 個以上（GF(2^8) の Lagrange 補間で f(0) を計算する）
pub fn combine(shares: &[(u8, u8)]) -> u8 {
    // シェアが 0 個の場合は panic させる（復元には最低 1 シェアが必要）
    assert!(!shares.is_empty(), "シェアが空では復元できない");
    // Lagrange 補間で f(0) を計算する（f(0) = secret）
    let mut secret: u8 = 0;
    // 各シェア i について Lagrange basis polynomial l_i(0) を計算する
    for i in 0..shares.len() {
        // シェア i の x 座標を取得する
        let xi = shares[i].0;
        // シェア i の y 座標（f(xi)）を取得する
        let yi = shares[i].1;
        // Lagrange basis l_i(0) = product_{j≠i} (0 - x_j) / (x_i - x_j) in GF(2^8) を計算する
        let mut basis: u8 = 1;
        // j ≠ i の全シェアについて積を計算する
        for j in 0..shares.len() {
            // i == j はスキップする（Lagrange 基底の定義上 j ≠ i のみ積を取る）
            if i == j {
                continue;
            }
            // j 番目のシェアの x 座標を取得する
            let xj = shares[j].0;
            // GF(2^8) では減算 = XOR（0 - x_j = x_j in GF(2^8)）なので numerator は x_j
            let num = xj;
            // x_i XOR x_j が denominator（GF(2^8) での差）
            let den = xi ^ xj;
            // den の逆元を計算して乗算することで除算を実現する
            basis = gf_mul(basis, gf_mul(num, gf_inv(den)));
        }
        // y_i * l_i(0) を XOR して f(0) に足し込む（GF(2^8) での加算）
        secret ^= gf_mul(yi, basis);
    }
    // 復元したシークレット（= f(0)）を返す
    secret
}

// テスト: split/combine の round-trip を検証するユニットテスト群
#[cfg(test)]
mod tests {
    // 親モジュールの全シンボルをインポートする
    use super::*;

    // split → combine で元のシークレットが復元できることを確認する（round-trip テスト）
    #[test]
    fn test_shamir_split_combine_roundtrip() {
        // テスト用シークレット値（任意の 1 バイト値）
        let secret: u8 = 0xAB;
        // 5 シェアに分割し threshold=3 で生成する
        let shares = split(secret, 3, 5);
        // 最初の 3 シェアを使って復元する
        let recovered = combine(&shares[..3]);
        // 元のシークレットと一致することを確認する
        assert_eq!(secret, recovered, "round-trip で復元値が一致しない");
    }

    // threshold=3 で異なる 3 シェアの組み合わせでも復元できることを確認する
    #[test]
    fn test_shamir_threshold_3() {
        // テスト用シークレット値
        let secret: u8 = 0x42;
        // threshold=3, total=5 でシェアを生成する
        let shares = split(secret, 3, 5);
        // 組み合わせ 0,1,2 でシェアを取得して復元する
        let recovered_012 = combine(&[shares[0], shares[1], shares[2]]);
        // 組み合わせ 0,2,4 でシェアを取得して復元する
        let recovered_024 = combine(&[shares[0], shares[2], shares[4]]);
        // 組み合わせ 1,3,4 でシェアを取得して復元する
        let recovered_134 = combine(&[shares[1], shares[3], shares[4]]);
        // 全ての組み合わせで同じシークレットが得られることを確認する
        assert_eq!(secret, recovered_012, "組み合わせ 0,1,2 で復元値が一致しない");
        // 組み合わせ 0,2,4 の検証をする
        assert_eq!(secret, recovered_024, "組み合わせ 0,2,4 で復元値が一致しない");
        // 組み合わせ 1,3,4 の検証をする
        assert_eq!(secret, recovered_134, "組み合わせ 1,3,4 で復元値が一致しない");
    }

    // GF 乗算の単体テスト（0 との乗算は 0 になることを確認する）
    #[test]
    fn test_gf_mul_by_zero() {
        // 任意の値と 0 の積は 0 であることを確認する
        assert_eq!(gf_mul(0xAB, 0), 0, "GF(2^8) での 0 との積は 0 でなければならない");
    }

    // GF 乗算の単体テスト（1 との乗算は元の値になることを確認する）
    #[test]
    fn test_gf_mul_by_one() {
        // 任意の値と 1 の積は元の値であることを確認する（乗法単位元）
        assert_eq!(gf_mul(0xCD, 1), 0xCD, "GF(2^8) での 1 との積は元の値でなければならない");
    }

    // GF 逆元の単体テスト（a * gf_inv(a) = 1 であることを確認する）
    #[test]
    fn test_gf_inv_product() {
        // テスト用の値を複数確認する
        for a in [1u8, 2, 5, 10, 0xFF, 0xAB] {
            // a と gf_inv(a) の積が 1 であることを確認する
            let inv_a = gf_inv(a);
            // 積を計算する
            let product = gf_mul(a, inv_a);
            // 積が 1（乗法単位元）であることを確認する
            assert_eq!(product, 1, "a={:#04x}: a * gf_inv(a) が 1 にならない（got {:#04x}）", a, product);
        }
    }
}
